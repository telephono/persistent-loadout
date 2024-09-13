use std::path::PathBuf;
use xplm::flight_loop::FlightLoopCallback;

use crate::debugln;
use crate::loadout::Data;

pub struct FlightLoopHandler {
    pub acf_livery_path: Option<PathBuf>,
}

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        if let Some(acf_livery_path) = &self.acf_livery_path {
            if let Err(e) = Data::restore_loadout_for_livery(acf_livery_path) {
                debugln!("{e}");
            }
        }

        // We are done...
        state.deactivate();
    }
}
