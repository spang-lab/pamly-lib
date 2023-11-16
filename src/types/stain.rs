use anyhow::{bail, Result};
use pyo3::{pyclass, pymethods, PyResult};
use std::convert::TryFrom;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, EnumString)]
#[repr(u8)]
#[pyclass]
#[rustfmt::skip]
#[strum(ascii_case_insensitive)]
pub enum Stain {
    Unknown = 0,
    HE = 1,
    CD3 = 3,
    CD20 = 20,
    CD30 = 30,
    CD68 = 68,
}

impl Stain {
    pub fn from(s: &str) -> Result<Stain> {
        let clean = s.replace(" ", "").replace("_", "").replace("&", "");
        match Stain::try_from(clean.as_str()) {
            Ok(l) => Ok(l),
            Err(e) => bail!(e.to_string()),
        }
    }
}

#[pymethods]
impl Stain {
    #[staticmethod]
    pub fn list() -> Vec<Stain> {
        Stain::iter().collect()
    }
    pub fn to_string(&self) -> String {
        let s = format!("{}", self);
        s
    }
    #[new]
    pub fn new(s: &str) -> PyResult<Stain> {
        let s = Stain::from(s)?;
        Ok(s)
    }
}

impl TryFrom<u8> for Stain {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Stain> {
        let labels = Stain::list();
        for label in labels {
            if label as u8 == value {
                return Ok(label);
            }
        }
        bail!("No stain for number {}", value);
    }
}

#[cfg(test)]
mod tests {
    use super::{Result, Stain, TryFrom};
    #[test]
    fn stain_consistency() -> Result<()> {
        for stain in Stain::list() {
            let num = stain as u8;
            let stain2 = Stain::try_from(num)?;
            assert_eq!(stain, stain2);
        }
        Ok(())
    }
    #[test]
    fn he() -> Result<()> {
        let he1 = Stain::from("HE")?;
        let he2 = Stain::from("H&E")?;
        let he3 = Stain::from("H_E")?;
        let he4 = Stain::from("H_E")?;
        assert_eq!(he1, he2);
        assert_eq!(he2, he3);
        assert_eq!(he3, he4);
        Ok(())
    }
}
