// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use std::cell::RefCell;
use std::ffi::{CStr, OsString};
use std::fmt::Display;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;

use thiserror::Error;

use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, StringRead};
use xplm::flight_loop::FlightLoop;
use xplm::plugin::{Plugin, PluginInfo};

use crate::flight_loop::FlightLoopHandler;
use crate::loadout::LoadoutFile;

pub static NAME: &str = concat!("Persistent Loadout v", env!("CARGO_PKG_VERSION"));
static SIGNATURE: &str = concat!("com.x-plane.xplm.", env!("CARGO_PKG_NAME"));
static DESCRIPTION: &str = "Persistent Loadout for Shenshee's B720";

// Build output path from these components
pub static XPLANE_OUTPUT_PATH: &str = "Output";
pub static PLUGIN_OUTPUT_PATH: &str = "B720";
pub static LOADOUT_FILENAME: &str = "persistent-loadout.json";

thread_local! {
    pub static GLOBAL_LIVERY: RefCell<PathBuf> = RefCell::new(PathBuf::new());
}

#[derive(Error, Debug)]
pub enum PluginError {
    // Capture other errors...
    #[error(transparent)]
    InputOutput(#[from] std::io::Error),
    #[error(transparent)]
    Utf8Error(#[from] std::str::Utf8Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FindError(#[from] xplm::data::borrowed::FindError),

    // Add our own errors...
    #[error("{NAME} aircraft {0:?} not supported")]
    AircraftNotSupported(String),
    #[error("{NAME} no path to {:?}", LOADOUT_FILENAME)]
    MissingPath,
    #[error("{NAME} detected startup with engines running")]
    StartupWithEnginesRunning,
}

pub struct PersistentLoadoutPlugin {
    handler: FlightLoop,
}

impl Plugin for PersistentLoadoutPlugin {
    type Error = PluginError;

    fn start() -> Result<Self, Self::Error> {
        debugln!("{NAME} starting up");

        let plugin = Self {
            handler: FlightLoop::new(FlightLoopHandler),
        };

        Ok(plugin)
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        // We only enable our plugin, if the aircraft is supported.
        let acf_icao: DataRef<[u8]> = DataRef::find("sim/aircraft/view/acf_ICAO")?;
        let acf_icao = acf_icao.get_as_string()?;
        if acf_icao != "B720" {
            return Err(PluginError::AircraftNotSupported(acf_icao));
        }

        // We only enable our plugin, if the user did startup the aircraft in cold & dark,
        // otherwise we return an error and the plugin remains disabled.
        let startup_running: DataRef<i32> = DataRef::find("sim/operation/prefs/startup_running")?;
        if startup_running.get() != 0 {
            return Err(PluginError::StartupWithEnginesRunning);
        }

        // After enabling our plugin, we need to wait for the flight loop to start,
        // so our datarefs are ready and accessible.
        debugln!("{NAME} enabling flight loop callback");
        self.handler.schedule_after_loops(60);

        Ok(())
    }

    fn disable(&mut self) {
        // When the plugin gets disabled (aka the sim shuts down or the user selects another
        // aircraft) we save the current loadout...
        let loadout = match LoadoutFile::from_current_livery() {
            Ok(loadout) => loadout,
            Err(error) => {
                debugln!("{NAME} something went wrong: {error}");
                self.handler.deactivate();
                return;
            }
        };

        if let Err(error) = loadout.save_loadout() {
            debugln!("something went wrong: {error}");
            self.handler.deactivate();
            return;
        }

        debugln!("{NAME} disabling");
        self.handler.deactivate();
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from(NAME),
            signature: String::from(SIGNATURE),
            description: String::from(DESCRIPTION),
        }
    }

    fn receive_message(&mut self, _from: i32, message: u32, param: *mut c_void) {
        if message == xplm_sys::XPLM_MSG_LIVERY_LOADED {
            // We are only interested in our own livery
            let index = param as usize;
            if index != 0 {
                return;
            }

            debugln!("{NAME} XPLM_MSG_LIVERY_LOADED");

            GLOBAL_LIVERY.with(|path| {
                let old_livery = (*path.borrow()).clone();

                // Ignore on first run...
                if old_livery.as_os_str().is_empty() {
                    return;
                }

                // Get new livery path
                let new_livery = match get_acf_livery_path() {
                    Ok(path) => path,
                    Err(error) => {
                        debugln!("{NAME} something went wrong: {error}");
                        return;
                    }
                };

                // Compare old and new livery path
                // Nothing to do if they are the same...
                if old_livery.as_os_str() == new_livery.as_os_str() {
                    return;
                }

                debugln!("{NAME} livery change detected");

                let old_loadout = match LoadoutFile::from_custom_livery(old_livery.as_os_str()) {
                    Ok(loadout) => loadout,
                    Err(error) => {
                        debugln!("{NAME} something went wrong: {error}");
                        return;
                    }
                };

                debugln!(
                    "{NAME} saving loadout for old livery {}",
                    old_livery.as_path().display()
                );

                if let Err(error) = old_loadout.save_loadout() {
                    debugln!("{NAME} something went wrong: {error}");
                    return;
                }

                let new_loadout = match LoadoutFile::from_custom_livery(new_livery.as_os_str()) {
                    Ok(loadout) => loadout,
                    Err(error) => {
                        debugln!("{NAME} something went wrong: {error}");
                        return;
                    }
                };

                debugln!(
                    "{NAME} restoring loadout for new livery {}",
                    new_livery.as_path().display()
                );

                if let Err(error) = new_loadout.restore_loadout() {
                    debugln!("{NAME} something went wrong: {error}");
                    return;
                };

                // Update "old"" livery
                *path.borrow_mut() = new_livery;
            });
        }
    }
}

/// XPLMGetNthAircraftModel wrapper
#[derive(Debug)]
pub struct AircraftModel {
    out_file: PathBuf,
    out_path: PathBuf,
}

impl AircraftModel {
    pub fn new(index: c_int) -> Result<Self, PluginError> {
        let mut out_file_buf: [c_char; 256] = [b'\0' as c_char; 256];
        let mut out_path_buf: [c_char; 512] = [b'\0' as c_char; 512];
        let out_file: &CStr;
        let out_path: &CStr;

        unsafe {
            xplm_sys::XPLMGetNthAircraftModel(
                index as c_int,
                out_file_buf.as_mut_ptr(),
                out_path_buf.as_mut_ptr(),
            );

            out_file = CStr::from_ptr(out_file_buf.as_ptr());
            out_path = CStr::from_ptr(out_path_buf.as_ptr());
        }

        let out_file = PathBuf::from(out_file.to_str()?);
        let out_path = PathBuf::from(out_path.to_str()?);

        Ok(Self { out_file, out_path })
    }

    #[allow(dead_code)]
    /// Return aircraft's acf file name without .acf extension
    pub fn out_file_stem(&self) -> OsString {
        self.out_file.file_stem().unwrap_or_default().to_owned()
    }

    #[allow(dead_code)]
    /// Return path to aircraft's acf file
    /// The path is relative to the X-Plane root directory
    pub fn relative_out_path(&self) -> PathBuf {
        let mut is_relative = false;

        // Iterate through the full path until we get the `Aircraft` folder and then start
        // returning everything.
        // This way we should get a path relative to X-Plane's root directory.
        // Maybe this could be a use case for XPLMGetSystemPath...
        let mut out_path: PathBuf = self
            .out_path
            .iter()
            .filter(|&segment| {
                if segment == "Aircraft" {
                    is_relative = true;
                }
                is_relative
            })
            .collect();

        // Truncate file name from path
        out_path.pop();

        out_path
    }
}

impl Display for AircraftModel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let out_path_file = self.out_path.join(self.out_file.as_path());
        write!(f, "{}", out_path_file.to_string_lossy())
    }
}

pub fn get_acf_livery_path() -> Result<PathBuf, PluginError> {
    let acf_livery_path: DataRef<[u8]> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
    let acf_livery_path = acf_livery_path.get_as_string()?;

    let mut output_file_path = PathBuf::from(XPLANE_OUTPUT_PATH);
    output_file_path.push(PLUGIN_OUTPUT_PATH);

    // Build path from aircraft model
    let aircraft_model = AircraftModel::new(0)?;
    match aircraft_model.out_file_stem().to_string_lossy().as_ref() {
        "Boeing_720" => output_file_path.push("720"),
        "Boeing_720B" => output_file_path.push("720B"),
        _ => {
            debugln!(
                "{NAME} failed to get known aircraft model from {:?}",
                aircraft_model
            );
            let aircraft = aircraft_model.out_file_stem().to_string_lossy().to_string();
            return Err(PluginError::AircraftNotSupported(aircraft));
        }
    }

    if acf_livery_path.is_empty() {
        // Set up a valid livery path for default livery.
        output_file_path.push("Default");
    } else {
        // Set up a valid livery path.
        let acf_livery_path = PathBuf::from(acf_livery_path.as_str());
        if let Some(livery_path) = acf_livery_path.components().last() {
            output_file_path.push(livery_path)
        } else {
            debugln!(
                "{NAME} failed to extract livery folder from {:?}",
                acf_livery_path
            );
            return Err(PluginError::MissingPath);
        };
    }
    Ok(output_file_path)
}
