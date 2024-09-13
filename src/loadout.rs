use std::collections::{BTreeMap, HashMap};
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
    map: BTreeMap<String, Loadout>,
}

impl std::fmt::Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let data = serde_json::to_string_pretty(&self.map).unwrap_or_default();
        f.write_str(&data)
    }
}

impl Data {
    pub fn save_loadout_for_livery(livery: &Path) -> Result<(), PluginError> {
        let mut data_file = PathBuf::from(livery);
        data_file.push(DATA_FILE_NAME);

        let data_file = data_file.to_string_lossy().to_string();

        Self::from_file(&data_file)?
            .update_from_sim()?
            .write_to_file()?;

        Ok(())
    }

    pub fn restore_loadout_for_livery(livery: &Path) -> Result<(), PluginError> {
        let mut data_file = PathBuf::from(livery);
        data_file.push(DATA_FILE_NAME);

        let data_file = data_file.to_string_lossy().to_string();

        Self::from_file(&data_file)?.write_into_sim()?;
        Ok(())
    }

    fn from_file(path: &str) -> std::io::Result<Self> {
        let path = Path::new(path).to_path_buf();

        let map: BTreeMap<String, Loadout> = match path.try_exists() {
            Err(e) => return Err(e),
            Ok(false) => {
                debugln!("loadout file {} not found", path.to_string_lossy());

                BTreeMap::new()
            }
            Ok(true) => {
                debugln!("found loadout file {}", path.to_string_lossy());
                let file = File::open(Path::new(&path))?;
                let reader = BufReader::new(&file);

                let map: HashMap<String, Loadout> = serde_json::from_reader(reader)?;
                let sorted: BTreeMap<String, Loadout> = map
                    .into_iter()
                    .map(|(k, v)| (k.to_ascii_lowercase(), v))
                    .collect();

                sorted
            }
        };

        Ok(Self { path, map })
    }

    fn livery_name(&self) -> String {
        let livery_folder = self.path.parent().unwrap();
        let livery_os_str = livery_folder.file_name().unwrap_or_default();
        let livery = livery_os_str.to_string_lossy().to_string();

        livery
    }

    fn write_into_sim(self) -> Result<Self, PluginError> {
        let mut datarefs = BorrowedDataRefs::initialize()?;

        let livery = self.livery_name();

        if let Some(loadout) = self.map.get(&livery.to_ascii_lowercase()) {
            debugln!("found loadout for {livery}: {loadout}");
            datarefs.m_fuel.set(loadout.m_fuel.as_slice());
        };

        Ok(self)
    }

    fn update_from_sim(mut self) -> Result<Self, PluginError> {
        let datarefs = BorrowedDataRefs::initialize()?;

        let livery = self.livery_name();
        let m_fuel = datarefs.m_fuel.as_vec();

        let loadout = Loadout { m_fuel };

        debugln!("updating loadout for {livery}: {loadout}");
        self.map.insert(livery.to_ascii_lowercase(), loadout);

        Ok(self)
    }

    fn write_to_file(self) -> std::io::Result<Self> {
        debugln!(
            "writing loadout to file {}",
            self.path.to_string_lossy()
        );

        let json_data = serde_json::to_string_pretty(&self.map)?;

        let mut file = File::create(&self.path)?;
        file.write_all(json_data.as_bytes())?;

        Ok(self)
    }
}
