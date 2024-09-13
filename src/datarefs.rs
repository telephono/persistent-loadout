use std::fmt::{Debug, Formatter};
use std::path::Path;

use xplm::data::borrowed::{DataRef, FindError};
use xplm::data::{ArrayRead, ReadOnly, ReadWrite, StringRead};

pub struct BorrowedDataRefs {
    // Path of current livery. Ends in dir separator. WARNING: slow dataref, don't read a lot!
    pub acf_livery_path: DataRef<[u8], ReadOnly>,
    // Fuel Tank Weight for 9 tanks
    pub m_fuel: DataRef<[f32], ReadWrite>,
}

impl Debug for BorrowedDataRefs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorrowedDataRefs")
            .field(
                "acf_livery_path",
                &self.acf_livery_path.get_as_string().unwrap_or_default(),
            )
            .field("m_fuel", &self.m_fuel.as_vec())
            .finish()
    }
}

impl BorrowedDataRefs {
    pub fn initialize() -> Result<Self, FindError> {
        let datarefs = BorrowedDataRefs {
            acf_livery_path: DataRef::find("sim/aircraft/view/acf_livery_path")?,
            m_fuel: DataRef::find("sim/flightmodel/weight/m_fuel")?.writeable()?,
        };

        Ok(datarefs)
    }

    pub fn livery_name(&self) -> String {
        let livery_path = self.acf_livery_path.get_as_string().unwrap_or_default();
        let livery_os_str = Path::new(&livery_path).file_name().unwrap_or_default();
        let livery = livery_os_str.to_string_lossy().to_string();

        livery
    }
}
