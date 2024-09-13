use xplm::data::borrowed::DataRef;
use xplm::data::{ReadOnly, StringRead};
use xplm::flight_loop::FlightLoopCallback;

use crate::debugln;
use crate::loadout::LoadoutData;

pub struct FlightLoopHandler {
    pub acf_livery_path: Option<String>,
}

impl FlightLoopCallback for FlightLoopHandler {
    fn flight_loop(&mut self, state: &mut xplm::flight_loop::LoopState) {
        let acf_livery_path: DataRef<[u8], ReadOnly> = DataRef::find("sim/aircraft/view/acf_livery_path").unwrap();
        let acf_livery_path = acf_livery_path.get_as_string().unwrap_or_default();

        if !acf_livery_path.is_empty() {
            self.acf_livery_path = Some(acf_livery_path);
        }

        if let Some(acf_livery_path) = &self.acf_livery_path {
            if let Err(e) = LoadoutData::restore_loadout(acf_livery_path) {
                debugln!("{e}");
            }
        }

        // We are done...
        state.deactivate();
    }
}
