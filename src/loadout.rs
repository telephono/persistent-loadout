use std::fs::File;
use std::io::{BufReader, Write};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use xplm::data::{ArrayRead, ArrayReadWrite};

use crate::datarefs::BorrowedDataRefs;
use crate::debugln;
use crate::plugin::{PluginError, DATA_FILE_NAME};

#[derive(Default, Serialize, Deserialize)]
pub struct Loadout {
    pub m_fuel: Vec<f32>,
}

impl std::fmt::Display for Loadout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("fuel tanks: {:?}", self.m_fuel))
    }
}

pub struct Data {
    path: PathBuf,
    loadout: Option<Loadout>,
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = serde_json::to_string_pretty(&self.loadout).unwrap_or_default();
        f.write_str(&data)
    }
}

impl Data {
    pub fn save_loadout(livery_path: &str) -> Result<(), PluginError> {
        let mut data_file = PathBuf::from(livery_path);
        data_file.push(DATA_FILE_NAME);

        let data_file = data_file.to_string_lossy().to_string();

        Self::from_file(&data_file)?
            .update_from_sim()?
            .write_into_file()?;

        Ok(())
    }

    pub fn restore_loadout(livery_path: &str) -> Result<(), PluginError> {
        let mut data_file = PathBuf::from(livery_path);
        data_file.push(DATA_FILE_NAME);

        let data_file = data_file.to_string_lossy().to_string();
        Self::from_file(&data_file)?.write_into_sim()?;

        Ok(())
    }

    fn from_file(path: &str) -> std::io::Result<Self> {
        let path = Path::new(path).to_path_buf();

        let loadout: Option<Loadout> = match path.try_exists() {
            Err(e) => return Err(e),
            Ok(false) => {
                debugln!("loadout file {} not found", path.to_string_lossy());
                None
            }
            Ok(true) => {
                debugln!("found loadout file {}", path.to_string_lossy());
                let file = File::open(Path::new(&path))?;
                let reader = BufReader::new(&file);
                Some(serde_json::from_reader(reader)?)
            }
        };

        Ok(Self { path, loadout })
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
            debugln!("writing loadout to file {}", self.path.to_string_lossy());
            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(&self.path)?;
            file.write_all(json_data.as_bytes())?;
        }

        Ok(self)
    }
}
