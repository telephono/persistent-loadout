use std::fs::File;
use std::io::{BufReader, Write};
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use xplm::data::borrowed::DataRef;
use xplm::data::{ArrayRead, ArrayReadWrite, ReadOnly, ReadWrite, StringRead};

use crate::debugln;
use crate::plugin::{PluginError, DATA_FILE_NAME};

#[derive(Default, Serialize, Deserialize)]
struct Loadout {
    m_fuel: Vec<f32>,
}

impl std::fmt::Display for Loadout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(format!("fuel tanks: {:?}", self.m_fuel).as_str())
    }
}

pub struct LoadoutData {
    data_file: PathBuf,
    loadout: Option<Loadout>,
}

impl std::fmt::Display for LoadoutData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = serde_json::to_string_pretty(&self.loadout).unwrap_or_default();
        f.write_str(data.as_str())
    }
}

impl LoadoutData {
    pub fn save_loadout() -> Result<(), PluginError> {
        let acf_livery_path: DataRef<[u8], ReadOnly> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
        let acf_livery_path = acf_livery_path.get_as_string().unwrap_or_default();

        Self::from_file(acf_livery_path.as_str())?
            .update_from_sim()?
            .write_into_file()?;

        Ok(())
    }

    pub fn restore_loadout() -> Result<(), PluginError> {
        let acf_livery_path: DataRef<[u8], ReadOnly> = DataRef::find("sim/aircraft/view/acf_livery_path")?;
        let acf_livery_path = acf_livery_path.get_as_string().unwrap_or_default();

        Self::from_file(acf_livery_path.as_str())?
            .write_into_sim()?;

        Ok(())
    }

    fn from_file(acf_livery_path: &str) -> std::io::Result<Self> {
        let mut data_file = PathBuf::from(acf_livery_path);
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
                let loadout = serde_json::from_reader(reader)?;
                Some(loadout)
            }
        };

        Ok(Self { data_file, loadout })
    }

    fn update_from_sim(mut self) -> Result<Self, PluginError> {
        debugln!("getting loadout from X-Plane");

        let m_fuel: DataRef<[f32], ReadOnly> = DataRef::find("sim/flightmodel/weight/m_fuel")?;
        let loadout = Loadout { m_fuel: m_fuel.as_vec() };

        self.loadout = Some(loadout);

        Ok(self)
    }

    fn write_into_sim(self) -> Result<Self, PluginError> {
        debugln!("setting loadout in X-Plane");

        if let Some(loadout) = self.loadout.as_ref() {
            let mut m_fuel: DataRef<[f32], ReadWrite> = DataRef::find("sim/flightmodel/weight/m_fuel")?
                .writeable()?;

            m_fuel.set(loadout.m_fuel.as_slice());
        }

        Ok(self)
    }

    fn write_into_file(self) -> std::io::Result<Self> {
        if let Some(loadout) = self.loadout.as_ref() {
            debugln!("writing loadout to file {}", self.data_file.to_string_lossy());

            let json_data = serde_json::to_string_pretty(loadout)?;
            let mut file = File::create(self.data_file.as_os_str())?;
            file.write_all(json_data.as_bytes())?;
        }

        Ok(self)
    }
}
