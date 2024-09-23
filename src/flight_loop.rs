use xplm::flight_loop::FlightLoopCallback;

use crate::loadout::LoadoutData;
use crate::plugin::NAME;

pub struct FlightLoopHandler;

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        // In our flight loop callback, our datarefs should be ready, and we can read the loadout
        // from file and restore it into the sim.
        if let Err(error) = LoadoutData::restore_loadout() {
            debugln!("{NAME} something went wrong: {error}");
        }

        // We are done. We don't need to run our flight loop callback anymore...
        state.deactivate();
    }
}
