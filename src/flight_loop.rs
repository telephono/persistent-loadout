use xplm::flight_loop::FlightLoopCallback;

use crate::debugln;
use crate::loadout::LoadoutData;

pub struct FlightLoopHandler;

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        if let Err(e) = LoadoutData::restore_loadout() {
            debugln!("{e}");
        }

        // We are done...
        state.deactivate();
    }
}
