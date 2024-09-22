use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadOnly, ReadWrite, StringRead};

use crate::debugln;
use crate::plugin::{PluginError, LOADOUT_FILENAME};

#[derive(Serialize, Deserialize)]
struct Loadout {
    m_fuel: Vec<f32>,
    autobrake: f32,
    autothrottle: f32,
    navigation: f32,
}

pub struct LoadoutData {
    file: PathBuf,
    loadout: Option<Loadout>,
}

impl LoadoutData {
    pub fn save_loadout() -> Result<(), PluginError> {
        Self::from_file()?
            .get_from_sim()?
            .write_into_file()?;

        Ok(())
    }

    pub fn restore_loadout() -> Result<(), PluginError> {
        Self::from_file()?
            .write_into_sim()?;

        Ok(())
    }

    fn from_file() -> Result<Self, PluginError> {
        let acf_livery_path: DataRef<[u8], ReadOnly> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
        let acf_livery_path = acf_livery_path.get_as_string()?;

        let mut loadout_file = PathBuf::from(acf_livery_path);
        loadout_file.push(LOADOUT_FILENAME);

        let loadout: Option<Loadout> = match loadout_file.try_exists() {
            Err(error) => return Err(error.into()),

            Ok(false) => {
                debugln!("no loadout file {} found", loadout_file.to_string_lossy());
                None
            }

            Ok(true) => {
                debugln!("found loadout file {}", loadout_file.to_string_lossy());
                let file = File::open(&loadout_file)?;
                let reader = BufReader::new(&file);
                let loadout = serde_json::from_reader(reader)?;
                Some(loadout)
            }
        };

        Ok(Self { file: loadout_file, loadout })
    }

    fn get_from_sim(mut self) -> Result<Self, PluginError> {
        debugln!("getting loadout from X-Plane");
        let m_fuel: DataRef<[f32], ReadOnly> = DataRef::find("sim/flightmodel/weight/m_fuel")?;
        let generic_lights_switch: DataRef<[f32], ReadOnly> = DataRef::find("sim/cockpit2/switches/generic_lights_switch")?;

        let generic_lights_switch = generic_lights_switch.as_vec();

        let autothrottle = match generic_lights_switch.get(49) {
            Some(&autthrottle) => autthrottle,
            None => 0.0,
        };

        let autobrake = match generic_lights_switch.get(50) {
            Some(&autobrake) => autobrake,
            None => 0.0
        };

        let navigation = match generic_lights_switch.get(84) {
            Some(&navigation) => navigation,
            None => 0.0,
        };

        let loadout = Loadout {
            m_fuel: m_fuel.as_vec(),
            autobrake,
            autothrottle,
            navigation,
        };

        self.loadout = Some(loadout);
        Ok(self)
    }

    fn write_into_sim(self) -> Result<Self, PluginError> {
        if let Some(loadout) = self.loadout.as_ref() {
            debugln!("setting loadout in X-Plane");
            let mut m_fuel: DataRef<[f32], ReadWrite> = DataRef::find("sim/flightmodel/weight/m_fuel")?
                .writeable()?;
            let mut generic_lights_switch: DataRef<[f32], ReadWrite> = DataRef::find("sim/cockpit2/switches/generic_lights_switch")?
                .writeable()?;

            // Write equipment into sim...
            let mut new_generic_lights_switch = generic_lights_switch.as_vec();
            if new_generic_lights_switch.len() >= 128 {
                new_generic_lights_switch[49] = loadout.autothrottle;
                new_generic_lights_switch[50] = loadout.autobrake;
                new_generic_lights_switch[84] = loadout.navigation;
                generic_lights_switch.set(new_generic_lights_switch.as_slice());
            } else {
                debugln!("sim/cockpit2/switches/generic_lights_switch did not return at least 128 entries, skipping equipment setup");
            }

            // Write fuel levels into sim...
            let new_m_fuel = loadout.m_fuel.as_slice();
            m_fuel.set(new_m_fuel);
        }

        Ok(self)
    }

    fn write_into_file(self) -> std::io::Result<Self> {
        if let Some(loadout) = self.loadout.as_ref() {
            debugln!("writing loadout to file {}", self.file.to_string_lossy());
            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(self.file.as_os_str())?;
            file.write_all(json_data.as_bytes())?;
        }

        Ok(self)
    }
}
