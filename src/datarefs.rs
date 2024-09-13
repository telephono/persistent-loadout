use std::fmt::{Debug, Formatter};

use xplm::data::borrowed::{DataRef, FindError};
use xplm::data::{ArrayRead, ReadWrite};

pub struct BorrowedDataRefs {
    // Fuel Tank Weight for 9 tanks
    pub m_fuel: DataRef<[f32], ReadWrite>,
}

impl Debug for BorrowedDataRefs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BorrowedDataRefs")
            .field("m_fuel", &self.m_fuel.as_vec())
            .finish()
    }
}

impl BorrowedDataRefs {
    pub fn initialize() -> Result<Self, FindError> {
        let datarefs = BorrowedDataRefs {
            m_fuel: DataRef::find("sim/flightmodel/weight/m_fuel")?.writeable()?,
        };

        Ok(datarefs)
    }
}
