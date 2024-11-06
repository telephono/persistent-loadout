// Copyright (c) 2024 telephono
// Licensed under the MIT License. See LICENSE file in the project root for full license information.

use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadWrite, StringRead};

use crate::plugin::{AircraftModel, PluginError};
use crate::plugin::{LOADOUT_FILENAME, NAME, PLUGIN_OUTPUT_PATH, XPLANE_OUTPUT_PATH};

// Light switch indices for different equipment configururations
const AUTOTHROTTLE: usize = 49;
const AUTOBRAKE: usize = 50;
const HF_ANTENNA: usize = 56;
const NAVIGATION: usize = 84;

#[derive(Serialize, Deserialize)]
struct Loadout {
    m_fuel: Vec<f32>,
    #[serde(default)]
    autobrake: f32,
    #[serde(default)]
    autothrottle: f32,
    #[serde(default)]
    hf_antenna: f32,
    #[serde(default)]
    navigation: f32,
}

pub struct LoadoutData {
    file: Option<PathBuf>,
    loadout: Option<Loadout>,
}

impl LoadoutData {
    /// Read loadout from sim and write it into a JSON file.
    pub fn save_loadout() -> Result<(), PluginError> {
        Self::new()?.loadout_from_sim()?.write_into_file()?;

        Ok(())
    }

    /// Read loadout from JSON file and write it into sim.
    pub fn restore_loadout() -> Result<(), PluginError> {
        Self::new()?.loadout_from_file()?.write_into_sim()?;

        Ok(())
    }

    fn new() -> Result<Self, PluginError> {
        let acf_livery_path: DataRef<[u8]> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
        let acf_livery_path = acf_livery_path.get_as_string()?;

        let mut output_file_path = PathBuf::from(XPLANE_OUTPUT_PATH);
        output_file_path.push(PLUGIN_OUTPUT_PATH);

        // Build path from aircraft model
        let aircraft_model = AircraftModel::new(0)?;
        match aircraft_model.out_file_stem().to_string_lossy().as_ref() {
            "Boeing_720" => output_file_path.push("720"),
            "Boeing_720B" => output_file_path.push("720B"),
            _ => {
                debugln!(
                    "{NAME} failed to get known aircraft model from {:?}",
                    aircraft_model
                );
                let aircraft = aircraft_model.out_file_stem().to_string_lossy().to_string();
                return Err(PluginError::AircraftNotSupported(aircraft));
            }
        }

        if acf_livery_path.is_empty() {
            // Set up a valid livery path for default livery.
            output_file_path.push("Default");
        } else {
            // Set up a valid livery path.
            let acf_livery_path = PathBuf::from(acf_livery_path.as_str());
            if let Some(livery_path) = acf_livery_path.components().last() {
                output_file_path.push(livery_path)
            } else {
                debugln!(
                    "{NAME} failed to extract livery folder from {:?}",
                    acf_livery_path
                );
                return Err(PluginError::MissingPath);
            };
        }

        output_file_path.push(LOADOUT_FILENAME);

        Ok(Self {
            file: Some(output_file_path),
            loadout: None,
        })
    }

    /// Read the loadout from a JSON file, but only if `self.file` is set.
    fn loadout_from_file(mut self) -> Result<Self, PluginError> {
        if let Some(file) = self.file.as_ref() {
            debugln!(
                "{NAME} reading loadout from file {:?}",
                file.to_string_lossy()
            );

            self.loadout = match file.try_exists() {
                Err(error) => return Err(error.into()),

                Ok(false) => {
                    debugln!("{NAME} loadout file {:?} not found", file.to_string_lossy());
                    None
                }

                Ok(true) => {
                    // Read JSON into a String first, since this should be faster than
                    // using `serde_json::from_reader`
                    // see: https://github.com/serde-rs/json/issues/160
                    let mut f = File::open(file)?;
                    let mut buffer = String::new();

                    f.read_to_string(&mut buffer)?;

                    // Parse JSON, but return None if there was a parsing error...
                    serde_json::from_str(&buffer).unwrap_or_else(|error| {
                        debugln!("{NAME} could not parse loadout: {error}");
                        None
                    })
                }
            };
        } else {
            return Err(PluginError::MissingPath);
        }

        Ok(self)
    }

    fn loadout_from_sim(mut self) -> Result<Self, PluginError> {
        let m_fuel: DataRef<[f32]> = DataRef::find("sim/flightmodel/weight/m_fuel")?;
        let generic_lights_switch: DataRef<[f32]> =
            DataRef::find("sim/cockpit2/switches/generic_lights_switch")?;

        let generic_lights_switch = generic_lights_switch.as_vec();

        let autothrottle = generic_lights_switch
            .get(AUTOTHROTTLE)
            .copied()
            .unwrap_or_default();
        let autobrake = generic_lights_switch
            .get(AUTOBRAKE)
            .copied()
            .unwrap_or_default();
        let hf_antenna = generic_lights_switch
            .get(HF_ANTENNA)
            .copied()
            .unwrap_or_default();
        let navigation = generic_lights_switch
            .get(NAVIGATION)
            .copied()
            .unwrap_or_default();

        self.loadout = Some(Loadout {
            m_fuel: m_fuel.as_vec(),
            autobrake,
            autothrottle,
            hf_antenna,
            navigation,
        });

        Ok(self)
    }

    fn write_into_sim(self) -> Result<Self, PluginError> {
        if let Some(loadout) = self.loadout.as_ref() {
            // Write fuel levels into sim...
            let mut m_fuel: DataRef<[f32], ReadWrite> =
                DataRef::find("sim/flightmodel/weight/m_fuel")?.writeable()?;
            m_fuel.set(loadout.m_fuel.as_slice());

            // Write equipment config into sim...
            let mut generic_lights_switch: DataRef<[f32], ReadWrite> =
                DataRef::find("sim/cockpit2/switches/generic_lights_switch")?.writeable()?;

            let new_generic_lights_switch: Vec<f32> = generic_lights_switch
                .as_vec()
                .into_iter()
                .enumerate()
                .map(|(idx, value)| match idx {
                    AUTOTHROTTLE => loadout.autothrottle,
                    AUTOBRAKE => loadout.autobrake,
                    HF_ANTENNA => loadout.hf_antenna,
                    NAVIGATION => loadout.navigation,
                    _ => value,
                })
                .collect();

            generic_lights_switch.set(new_generic_lights_switch.as_slice());
        }

        Ok(self)
    }

    fn write_into_file(self) -> Result<Self, PluginError> {
        if let (Some(file), Some(loadout)) = (self.file.as_ref(), self.loadout.as_ref()) {
            // Check if path to loadout file exists, create it otherwise
            if let Some(file_path) = file.parent() {
                if !file_path.try_exists()? {
                    debugln!("{NAME} creating directory for livery {:?}", file_path);
                    // TODO:
                    // Don't try to create the directory just yet. We need to verify it first...
                    std::fs::create_dir_all(file_path)?;
                }
            };

            debugln!(
                "{NAME} writing loadout into file {:?}",
                file.to_string_lossy()
            );
            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(file.as_path())?;
            file.write_all(json_data.as_bytes())?;
        } else {
            return Err(PluginError::MissingPath);
        }

        Ok(self)
    }
}
