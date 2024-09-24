// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use thiserror::Error;

use xplm::data::borrowed::DataRef;
use xplm::data::{DataRead, StringRead};
use xplm::flight_loop::FlightLoop;
use xplm::plugin::{Plugin, PluginInfo};

use crate::flight_loop::FlightLoopHandler;
use crate::loadout::LoadoutData;

pub static NAME: &str = concat!("Persistent Loadout v", env!("CARGO_PKG_VERSION"));
static SIGNATURE: &str = concat!("com.x-plane.xplm.", env!("CARGO_PKG_NAME"));
static DESCRIPTION: &str = "Persistent loadout for X-Plane";
pub static LOADOUT_FILENAME: &str = "persistent-loadout.json";

#[derive(Error, Debug)]
pub enum PluginError {
    #[error(transparent)]
    InputOutput(#[from] std::io::Error),
    #[error(transparent)]
    FromUtf8Error(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error(transparent)]
    FindError(#[from] xplm::data::borrowed::FindError),
    #[error("expected {dataref:?} with a length of {expected:?}, found {found:?}")]
    UnexpectedArrayLengthError {
        dataref: String,
        expected: usize,
        found: usize,
    },
    #[error("{NAME} aircraft with ICAO code {acf_icao:?} not supported")]
    AircraftNotSupported {
        acf_icao: String,
    },
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
            return Err(PluginError::AircraftNotSupported { acf_icao });
        }

        // We only enable our plugin, if the user did startup the aircraft in cold & dark,
        // otherwise we return an error and the plugin remains disabled.
        let startup_running: DataRef<i32> = DataRef::find("sim/operation/prefs/startup_running")?;
        if startup_running.get() != 0 {
            return Err(PluginError::StartupWithEnginesRunning);
        }

        debug!("{NAME} enabling flight loop callback... ");

        // After enabling our plugin, we need to wait for the flight loop to start,
        // so our datarefs are ready and accessible.
        self.handler.schedule_after_loops(60);

        debugln!("done");
        Ok(())
    }

    fn disable(&mut self) {
        // When the plugin gets disabled (aka the sim shuts down or the user selects another aircraft)
        // we save the current loadout...
        if let Err(error) = LoadoutData::save_loadout() {
            debugln!("something went wrong: {error}");
        }

        debug!("{NAME} disabling... ");
        self.handler.deactivate();
        debugln!("done");
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from(NAME),
            signature: String::from(SIGNATURE),
            description: String::from(DESCRIPTION),
        }
    }
}
