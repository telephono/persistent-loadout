use xplm::flight_loop::FlightLoopCallback;

use crate::debugln;
use crate::loadout::LoadoutData;

pub struct FlightLoopHandler;

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        if let Err(error) = LoadoutData::restore_loadout() {
            debugln!("{error}");
        }

        // We are done...
        state.deactivate();
    }
}
