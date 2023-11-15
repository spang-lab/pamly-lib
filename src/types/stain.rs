use anyhow::{bail, Result};
use pyo3::{pyclass, pymethods, PyResult};
use std::convert::TryFrom;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
#[pyclass]
pub enum Stain {
    UNKNOWN = 0,
    HE = 1,
    CD3 = 3,
    CD20 = 20,
    CD30 = 30,
    CD68 = 68,
}

#[pymethods]
impl Stain {
    #[staticmethod]
    pub fn list() -> Vec<Stain> {
        vec![
            Stain::UNKNOWN,
            Stain::HE,
            Stain::CD3,
            Stain::CD20,
            Stain::CD30,
            Stain::CD68,
        ]
    }
    pub fn to_string(&self) -> String {
        let s = format!("{}", self);
        s
    }
    #[new]
    pub fn new(s: &str) -> PyResult<Stain> {
        let diagnosis = Stain::try_from(s)?;
        Ok(diagnosis)
    }
}
impl TryFrom<&str> for Stain {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Stain> {
        match s {
            "HE" => Ok(Stain::HE),
            "CD20" => Ok(Stain::CD20),
            "CD3" => Ok(Stain::CD3),
            "CD68" => Ok(Stain::CD68),
            "CD30" => Ok(Stain::CD30),
            _ => bail!("Could not parse stain {}", s),
        }
    }
}

impl TryFrom<u8> for Stain {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Stain> {
        match v {
            0 => Ok(Stain::UNKNOWN),
            1 => Ok(Stain::HE),
            3 => Ok(Stain::CD3),
            20 => Ok(Stain::CD20),
            30 => Ok(Stain::CD30),
            68 => Ok(Stain::CD68),
            _ => bail!("Invalid stain number {}", v),
        }
    }
}

impl fmt::Display for Stain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Stain::UNKNOWN => write!(f, "UNKNOWN"),
            Stain::HE => write!(f, "H&E"),
            Stain::CD20 => write!(f, "CD20"),
            Stain::CD3 => write!(f, "CD3"),
            Stain::CD30 => write!(f, "CD30"),
            Stain::CD68 => write!(f, "CD68"),
        }
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
}
