use std::path::PathBuf;

use thiserror::Error;

use xplm::data::borrowed::{DataRef, FindError};
use xplm::data::{DataRead, ReadOnly, StringRead};
use xplm::flight_loop::FlightLoop;
use xplm::plugin::{Plugin, PluginInfo};

use crate::debugln;
use crate::flight_loop::FlightLoopHandler;
use crate::loadout::Data;

pub static NAME: &str = concat!("Persistent Loadout", " ", "v", env!("CARGO_PKG_VERSION"));
static SIGNATURE: &str = concat!("com.x-plane.xplm.", env!("CARGO_PKG_NAME"));
static DESCRIPTION: &str = "Persistent loadout for X-Plane";
pub static DATA_FILE_NAME: &str = "persistent-loadout.json";

#[derive(Error, Debug)]
pub enum PluginError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    FindDataRef(#[from] FindError),
    #[error("no cold and dark startup")]
    NoColdAndDarkStartup,
    #[error("could not get livery from acf_livery_path")]
    UnknownAcfLiveryPath,
}

pub struct PersistentLoadoutPlugin {
    handler: FlightLoop,
    acf_livery_path: Option<PathBuf>,
}

impl Plugin for PersistentLoadoutPlugin {
    type Error = PluginError;

    fn start() -> Result<Self, Self::Error> {
        debugln!("starting up...");

        let acf_livery_path: DataRef<[u8], ReadOnly> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
        let acf_livery_path = acf_livery_path.get_as_string().unwrap_or_default();

        if acf_livery_path.is_empty() {
            return Err(PluginError::UnknownAcfLiveryPath);
        }

        debugln!("acf_livery_path {}", acf_livery_path);

        let flight_loop_handler = FlightLoopHandler {
            acf_livery_path: Some(PathBuf::from(&acf_livery_path)),
        };

        let plugin = Self {
            handler: FlightLoop::new(flight_loop_handler),
            acf_livery_path: Some(PathBuf::from(&acf_livery_path)),
        };

        Ok(plugin)
    }

    fn enable(&mut self) -> Result<(), Self::Error> {
        let startup_running: DataRef<i32> = DataRef::find("sim/operation/prefs/startup_running")?;
        if startup_running.get() != 0 {
            return Err(PluginError::NoColdAndDarkStartup);
        }

        debugln!("enabled...");
        self.handler.schedule_after_loops(60);

        Ok(())
    }

    fn disable(&mut self) {
        if let Some(acf_livery_path) = &self.acf_livery_path {
            if let Err(e) = Data::save_loadout_for_livery(acf_livery_path) {
                debugln!("{e}");
            }
        }

        self.handler.deactivate();
        debugln!("{NAME} disabled...");
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            name: String::from(NAME),
            signature: String::from(SIGNATURE),
            description: String::from(DESCRIPTION),
        }
    }
}
