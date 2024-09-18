use thiserror::Error;

use xplm::data::borrowed::DataRef;
use xplm::data::DataRead;
use xplm::flight_loop::FlightLoop;
use xplm::plugin::{Plugin, PluginInfo};

use crate::debugln;
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
    #[error("no cold and dark startup")]
    NotColdAndDark,
}

pub struct PersistentLoadoutPlugin {
    handler: FlightLoop,
}

impl Plugin for PersistentLoadoutPlugin {
    type Error = PluginError;

    fn start() -> Result<Self, Self::Error> {
        debugln!("starting up");

        let plugin = Self {
            handler: FlightLoop::new(FlightLoopHandler),
        };

        Ok(plugin)
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        let startup_running: DataRef<i32> = DataRef::find("sim/operation/prefs/startup_running")?;
        if startup_running.get() != 0 {
            return Err(PluginError::NotColdAndDark);
        }

        debugln!("enabled");
        self.handler.schedule_after_loops(60);

        Ok(())
    }

    fn disable(&mut self) {
        if let Err(e) = LoadoutData::save_loadout() {
            debugln!("{e}");
        }

        self.handler.deactivate();
        debugln!("{NAME} disabled");
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from(NAME),
            signature: String::from(SIGNATURE),
            description: String::from(DESCRIPTION),
        }
    }
}
