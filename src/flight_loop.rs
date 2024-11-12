// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use xplm::flight_loop::FlightLoopCallback;

use crate::loadout::LoadoutFile;
use crate::plugin::NAME;

pub struct FlightLoopHandler;

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        // In our flight loop callback, our datarefs should be ready, and we can read the loadout
        // from file and restore it into the sim.
        let loadout = match LoadoutFile::new().with_acf_livery_path() {
            Ok(loadout) => loadout,
            Err(error) => {
                debugln!("{NAME} something went wrong: {error}");
                state.deactivate();
                return;
            }
        };

        if let Err(error) = loadout.restore_loadout() {
            debugln!("{NAME} something went wrong: {error}");
            state.deactivate();
            return;
        }

        // We are done. We don't need to run our flight loop callback anymore...
        debugln!("{NAME} disabling flight loop callback");
        state.deactivate();
    }
}
