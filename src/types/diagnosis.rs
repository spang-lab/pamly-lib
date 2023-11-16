use anyhow::{bail, Result};
use pyo3::{pyclass, pymethods, PyResult};
use std::convert::TryFrom;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, EnumString)]
#[repr(u8)]
#[pyclass]
#[rustfmt::skip]
#[strum(ascii_case_insensitive)]
pub enum Diagnosis {
    Unknown = 0,
    HL = 1,
    DLBCL = 2,
    CLL = 3,
    FL = 4,
    MCL = 5,
    LTDS = 6,
}

impl Diagnosis {
    pub fn from(s: &str) -> Result<Diagnosis> {
        let clean = s.replace(" ", "").replace("_", "");
        match Diagnosis::try_from(clean.as_str()) {
            Ok(l) => Ok(l),
            Err(e) => bail!(e.to_string()),
        }
    }
}

#[pymethods]
impl Diagnosis {
    #[staticmethod]
    pub fn list() -> Vec<Diagnosis> {
        Diagnosis::iter().collect()
    }
    pub fn to_string(&self) -> String {
        let s = format!("{}", self);
        s
    }
    #[new]
    pub fn new(s: &str) -> PyResult<Diagnosis> {
        let diagnosis = Diagnosis::from(s)?;
        Ok(diagnosis)
    }
}

impl TryFrom<u8> for Diagnosis {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Diagnosis> {
        let labels = Diagnosis::list();
        for label in labels {
            if label as u8 == value {
                return Ok(label);
            }
        }
        bail!("No diagnosis for number {}", value);
    }
}

#[cfg(test)]
mod tests {
    use super::{Diagnosis, Result, TryFrom};
    #[test]
    fn diagnosis_consistency() -> Result<()> {
        for d in Diagnosis::list() {
            let num = d as u8;
            let d2 = Diagnosis::try_from(num)?;
            assert_eq!(d, d2);
        }
        Ok(())
    }
}
