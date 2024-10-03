// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use std::ffi::{c_char, c_int, CStr, OsString};
use std::fmt::Display;
use std::path::PathBuf;

use thiserror::Error;

use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, StringRead};
use xplm::flight_loop::FlightLoop;
use xplm::plugin::{Plugin, PluginInfo};

use crate::flight_loop::FlightLoopHandler;
use crate::loadout::LoadoutData;

pub static NAME: &str = concat!("Persistent Loadout v", env!("CARGO_PKG_VERSION"));
static SIGNATURE: &str = concat!("com.x-plane.xplm.", env!("CARGO_PKG_NAME"));
static DESCRIPTION: &str = "Persistent Loadout for Shenshee's B720";
pub static LOADOUT_FILENAME: &str = "persistent-loadout.json";

#[derive(Error, Debug)]
pub enum PluginError {
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
    #[error("{NAME} aircraft with ICAO code {0:?} not supported")]
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
        if let Err(error) = LoadoutData::save_loadout() {
            debugln!("something went wrong: {error}");
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

    /// Return aircraft's acf file name without .acf extension
    pub fn out_file_stem(&self) -> OsString {
        self.out_file.file_stem().unwrap_or_default().to_owned()
    }

    /// Return path to aircraft's acf file
    /// The path is relative to X-Plane root
    pub fn relative_out_path(&self) -> PathBuf {
        // Turn `out_path` into a relative path starting with the `Aircraft` directory
        let mut relative = false;

        let mut out_path: PathBuf = self
            .out_path
            .iter()
            .filter(|&segment| {
                if segment == "Aircraft" {
                    relative = true;
                }
                relative
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
