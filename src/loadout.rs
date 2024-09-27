// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadWrite, StringRead};

use crate::plugin::{PluginError, LOADOUT_FILENAME, NAME};

#[derive(Serialize, Deserialize)]
struct Loadout {
    m_fuel: Vec<f32>,
    autobrake: f32,
    autothrottle: f32,
    hf_antenna: f32,
    navigation: f32,
}

pub struct LoadoutData {
    file: PathBuf,
    loadout: Option<Loadout>,
}

impl LoadoutData {
    /// Read loadout from sim and write it into a JSON file.
    pub fn save_loadout() -> Result<(), PluginError> {
        Self::from_file()?
            .get_from_sim()?
            .write_into_file()?;

        Ok(())
    }

    /// Read loadout from JSON file and write it into sim.
    pub fn restore_loadout() -> Result<(), PluginError> {
        Self::from_file()?
            .write_into_sim()?;

        Ok(())
    }

    fn from_file() -> Result<Self, PluginError> {
        let acf_livery_path: DataRef<[u8]> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
        let acf_livery_path = acf_livery_path.get_as_string()?;

        let mut loadout_file = PathBuf::from(acf_livery_path);
        loadout_file.push(LOADOUT_FILENAME);

        let loadout: Option<Loadout> = match loadout_file.try_exists() {
            Err(error) => return Err(error.into()),

            Ok(false) => {
                debugln!("{NAME} no loadout file {:?} found", loadout_file.to_string_lossy());
                None
            }

            Ok(true) => {
                // Read JSON into a String first, since this should be faster than
                // using `serde_json::from_reader`
                // see: https://github.com/serde-rs/json/issues/160
                let mut f = File::open(&loadout_file)?;
                let mut buffer = String::new();

                f.read_to_string(&mut buffer)?;

                // Parse JSON, but return None if there was a parsing error...
                serde_json::from_str(&buffer).unwrap_or_else(|error| {
                    debugln!("{NAME} could not parse loadout: {error}");
                    None
                })
            }
        };

        Ok(Self { file: loadout_file, loadout })
    }

    fn get_from_sim(mut self) -> Result<Self, PluginError> {
        let m_fuel: DataRef<[f32]> = DataRef::find("sim/flightmodel/weight/m_fuel")?;
        let generic_lights_switch: DataRef<[f32]> = DataRef::find("sim/cockpit2/switches/generic_lights_switch")?;

        let generic_lights_switch = generic_lights_switch.as_vec();

        let autothrottle = generic_lights_switch.get(49).copied().unwrap_or_default();
        let autobrake = generic_lights_switch.get(50).copied().unwrap_or_default();
        let hf_antenna = generic_lights_switch.get(56).copied().unwrap_or_default();
        let navigation = generic_lights_switch.get(84).copied().unwrap_or_default();

        let loadout = Loadout {
            m_fuel: m_fuel.as_vec(),
            autobrake,
            autothrottle,
            hf_antenna,
            navigation,
        };

        self.loadout = Some(loadout);
        Ok(self)
    }

    fn write_into_sim(self) -> Result<Self, PluginError> {
        if let Some(loadout) = self.loadout.as_ref() {
            debugln!("{NAME} reading loadout from file {:?}", self.file.to_string_lossy());

            // Write fuel levels into sim...
            let mut m_fuel: DataRef<[f32], ReadWrite> = DataRef::find("sim/flightmodel/weight/m_fuel")?
                .writeable()?;
            m_fuel.set(loadout.m_fuel.as_slice());

            // Write equipment config into sim...
            let mut generic_lights_switch: DataRef<[f32], ReadWrite> = DataRef::find("sim/cockpit2/switches/generic_lights_switch")?
                .writeable()?;

            let new_generic_lights_switch: Vec<f32> = generic_lights_switch.as_vec()
                .into_iter()
                .enumerate()
                .map(|(idx, value)| {
                    match idx {
                        49 => loadout.autothrottle,
                        50 => loadout.autobrake,
                        56 => loadout.hf_antenna,
                        84 => loadout.navigation,
                        _ => value,
                    }
                })
                .collect();

            generic_lights_switch.set(new_generic_lights_switch.as_slice());
        }

        Ok(self)
    }

    fn write_into_file(self) -> std::io::Result<Self> {
        if let Some(loadout) = self.loadout.as_ref() {
            debugln!("{NAME} writing loadout into file {:?}", self.file.to_string_lossy());

            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(self.file.as_os_str())?;
            file.write_all(json_data.as_bytes())?;
        }

        Ok(self)
    }
}
