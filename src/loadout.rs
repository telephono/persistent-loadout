use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use xplm::data::{ArrayRead, ArrayReadWrite};

use crate::datarefs::BorrowedDataRefs;
use crate::debugln;
use crate::plugin::{PluginError, DATA_FILE_NAME};

#[derive(Default, Serialize, Deserialize)]
struct Loadout {
    m_fuel: Vec<f32>,
}

impl std::fmt::Display for Loadout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("fuel tanks: {:?}", self.m_fuel))
    }
}

pub struct LoadoutData {
    data_file: PathBuf,
    loadout: Option<Loadout>,
}

impl std::fmt::Display for LoadoutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = serde_json::to_string_pretty(&self.loadout).unwrap_or_default();
        f.write_str(&data)
    }
}

impl LoadoutData {
    pub fn save_loadout(livery_path: &str) -> Result<(), PluginError> {
        Self::from_file(livery_path)?
            .update_from_sim()?
            .write_into_file()?;
        Ok(())
    }

    pub fn restore_loadout(livery_path: &str) -> Result<(), PluginError> {
        Self::from_file(livery_path)?.write_into_sim()?;
        Ok(())
    }

    fn from_file(livery_path: &str) -> std::io::Result<Self> {
        let mut data_file = PathBuf::from(livery_path);
        data_file.push(DATA_FILE_NAME);

        let loadout: Option<Loadout> = match data_file.try_exists() {
            Err(e) => return Err(e),
            Ok(false) => {
                debugln!("loadout file {} not found", data_file.to_string_lossy());
                None
            }
            Ok(true) => {
                debugln!("found loadout file {}", data_file.to_string_lossy());
                let file = File::open(&data_file)?;
                let reader = BufReader::new(&file);
                Some(serde_json::from_reader(reader)?)
            }
        };

        Ok(Self { data_file, loadout })
    }

    fn update_from_sim(mut self) -> Result<Self, PluginError> {
        let datarefs = BorrowedDataRefs::initialize()?;
        let m_fuel = datarefs.m_fuel.as_vec();

        let loadout = Loadout { m_fuel };
        self.loadout = Some(loadout);

        Ok(self)
    }

    fn write_into_sim(self) -> Result<Self, PluginError> {
        if let Some(loadout) = &self.loadout {
            let mut datarefs = BorrowedDataRefs::initialize()?;
            datarefs.m_fuel.set(loadout.m_fuel.as_slice());
        }

        Ok(self)
    }

    fn write_into_file(self) -> std::io::Result<Self> {
        if let Some(loadout) = &self.loadout {
            debugln!("writing loadout to file {}", self.data_file.to_string_lossy());
            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(&self.data_file)?;
            file.write_all(json_data.as_bytes())?;
        }

        Ok(self)
    }
}
