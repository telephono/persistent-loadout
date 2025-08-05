// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use std::ffi::CStr;
use std::os::raw::{c_char, c_int, c_void};
use std::path::PathBuf;
use std::sync::Mutex;

use thiserror::Error;

use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, StringRead};
use xplm::feature::find_feature;
use xplm::flight_loop::FlightLoop;
use xplm::plugin::messages::XPLM_MSG_LIVERY_LOADED;
use xplm::plugin::{Plugin, PluginInfo};

use crate::flight_loop::FlightLoopHandler;
use crate::loadout::LoadoutFile;

pub static NAME: &str = concat!("Persistent Loadout v", env!("CARGO_PKG_VERSION"));
static SIGNATURE: &str = concat!("io.github.telephono.", env!("CARGO_PKG_NAME"));
static DESCRIPTION: &str = "Persistent Loadout for Shenshee's B720";

// Build output path from these components
pub static XPLANE_OUTPUT_PATH: &str = "Output";
pub static PLUGIN_OUTPUT_PATH: &str = "B720";
pub static LOADOUT_FILENAME: &str = "persistent-loadout.json";

pub static LIVERY: Mutex<Option<PathBuf>> = Mutex::new(None);

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

        if let Some(feature) = find_feature("XPLM_USE_NATIVE_PATHS") {
            debugln!("{NAME} enabling XPLM_USE_NATIVE_PATHS feature");
            feature.set_enabled(true);
        }

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
        let loadout = match LoadoutFile::with_acf_livery_path() {
            Ok(loadout) => loadout,
            Err(error) => {
                debugln!("{NAME} something went wrong: {error}");
                self.handler.deactivate();
                return;
            }
        };

        if let Err(error) = loadout.save_loadout() {
            debugln!("{NAME} something went wrong: {error}");
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

    fn receive_message(&mut self, _from: i32, message: i32, param: *mut c_void) {
        if message == XPLM_MSG_LIVERY_LOADED {
            // We are only interested in our own aircraft (index = 0)
            let index = param as i32;
            if index != 0 {
                return;
            }

            if let Ok(mut mutex) = LIVERY.lock() {
                let old_livery_path = mutex.as_ref().cloned();

                // Ignore on first run...
                if old_livery_path.is_none() {
                    return;
                }

                // Get new livery path
                let new_livery_path = match LoadoutFile::acf_livery_path() {
                    Ok(path) => Some(path),
                    Err(error) => {
                        debugln!("{NAME} something went wrong: {error}");
                        return;
                    }
                };

                // Compare old and new livery path
                // Nothing to do if they are the same...
                if old_livery_path == new_livery_path {
                    return;
                }

                debugln!("{NAME} livery change detected");

                // Save loadout for old livery...
                let old_loadout = match LoadoutFile::with_livery_path(
                    old_livery_path.as_ref().unwrap().as_os_str(),
                ) {
                    Ok(loadout) => loadout,
                    Err(error) => {
                        debugln!("{NAME} something went wrong: {error}");
                        return;
                    }
                };

                if let Err(error) = old_loadout.save_loadout() {
                    debugln!("{NAME} something went wrong: {error}");
                    return;
                }

                // Restore loadout for new livery...
                let new_loadout = match LoadoutFile::with_livery_path(
                    new_livery_path.as_ref().unwrap().as_os_str(),
                ) {
                    Ok(loadout) => loadout,
                    Err(error) => {
                        debugln!("{NAME} something went wrong: {error}");
                        return;
                    }
                };

                if let Err(error) = new_loadout.restore_loadout() {
                    debugln!("{NAME} something went wrong: {error}");
                    return;
                };

                // Update "old" livery
                *mutex = new_livery_path;
            };
        }
    }
}

#[allow(dead_code)]
/// XPLMGetNthAircraftModel wrapper
#[derive(Debug)]
pub struct AircraftModel {
    pub out_file: PathBuf,
    pub out_path: PathBuf,
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
}
