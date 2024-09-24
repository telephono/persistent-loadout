// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadWrite, StringRead};

use crate::plugin::PluginError::UnexpectedArrayLengthError;
use crate::plugin::{PluginError, LOADOUT_FILENAME, NAME};

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
                let file = File::open(&loadout_file)?;
                let reader = BufReader::new(&file);

                // Parse JSON file, but return None if there was a parsing error...
                serde_json::from_reader(reader).unwrap_or_else(|error| {
                    debugln!("{NAME} could not parse loadout from file {:?}: {error}", loadout_file.to_string_lossy());
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

        // "sim/cockpit2/switches/generic_lights_switch" should be float[128]
        // If it isn't, bail out...
        if generic_lights_switch.len() < 128 {
            return Err(UnexpectedArrayLengthError {
                dataref: "sim/cockpit2/switches/generic_lights_switch".to_string(),
                expected: 128,
                found: generic_lights_switch.len(),
            });
        }

        let autothrottle = generic_lights_switch[49];
        let autobrake = generic_lights_switch[50];
        let navigation = generic_lights_switch[84];

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
            let mut m_fuel: DataRef<[f32], ReadWrite> = DataRef::find("sim/flightmodel/weight/m_fuel")?
                .writeable()?;
            let mut generic_lights_switch: DataRef<[f32], ReadWrite> = DataRef::find("sim/cockpit2/switches/generic_lights_switch")?
                .writeable()?;

            // Write equipment into sim...
            let mut new_generic_lights_switch = generic_lights_switch.as_vec();

            // "sim/cockpit2/switches/generic_lights_switch" should be float[128]
            // If it isn't, bail out...
            if generic_lights_switch.len() < 128 {
                return Err(UnexpectedArrayLengthError {
                    dataref: "sim/cockpit2/switches/generic_lights_switch".to_string(),
                    expected: 128,
                    found: generic_lights_switch.len(),
                });
            }

            new_generic_lights_switch[49] = loadout.autothrottle;
            new_generic_lights_switch[50] = loadout.autobrake;
            new_generic_lights_switch[84] = loadout.navigation;

            debug!("{NAME} reading loadout from file {:?}... ", self.file.to_string_lossy());

            // Write equipment config into sim...
            generic_lights_switch.set(new_generic_lights_switch.as_slice());

            // Write fuel levels into sim...
            let new_m_fuel = loadout.m_fuel.as_slice();
            m_fuel.set(new_m_fuel);

            debugln!("done");
        }

        Ok(self)
    }

    fn write_into_file(self) -> std::io::Result<Self> {
        if let Some(loadout) = self.loadout.as_ref() {
            debug!("{NAME} writing loadout into file {:?}... ", self.file.to_string_lossy());

            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(self.file.as_os_str())?;
            file.write_all(json_data.as_bytes())?;

            debugln!("done");
        }

        Ok(self)
    }
}
