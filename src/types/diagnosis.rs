use anyhow::{bail, Result};
use pyo3::{pyclass, pymethods, PyResult};
use std::convert::{Into, TryFrom};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
#[pyclass]
pub enum Diagnosis {
    UNKNOWN = 0,
    HL = 1,
    DLBCL = 2,
    CLL = 3,
    FL = 4,
    MCL = 5,
    LTDS = 6,
}

#[pymethods]
impl Diagnosis {
    #[staticmethod]
    pub fn list() -> Vec<Diagnosis> {
        vec![
            Diagnosis::UNKNOWN,
            Diagnosis::HL,
            Diagnosis::DLBCL,
            Diagnosis::CLL,
            Diagnosis::FL,
            Diagnosis::MCL,
            Diagnosis::LTDS,
        ]
    }
    pub fn to_string(&self) -> String {
        let s = format!("{}", self);
        s
    }
    #[new]
    pub fn new(s: &str) -> PyResult<Diagnosis> {
        let diagnosis = Diagnosis::try_from(s)?;
        Ok(diagnosis)
    }
}

impl TryFrom<&str> for Diagnosis {
    type Error = anyhow::Error;
    fn try_from(s: &str) -> Result<Diagnosis> {
        match s {
            "HL" => Ok(Diagnosis::HL),
            "DLBCL" => Ok(Diagnosis::DLBCL),
            "CLL" => Ok(Diagnosis::CLL),
            "FL" => Ok(Diagnosis::FL),
            "MCL" => Ok(Diagnosis::MCL),
            "Lts" => Ok(Diagnosis::LTDS),
            _ => bail!("Could not parse Diagnosis {}", s),
        }
    }
}
impl TryFrom<u8> for Diagnosis {
    type Error = anyhow::Error;
    fn try_from(v: u8) -> Result<Diagnosis> {
        match v {
            0 => Ok(Diagnosis::UNKNOWN),
            1 => Ok(Diagnosis::HL),
            2 => Ok(Diagnosis::DLBCL),
            3 => Ok(Diagnosis::CLL),
            4 => Ok(Diagnosis::FL),
            5 => Ok(Diagnosis::MCL),
            6 => Ok(Diagnosis::LTDS),
            _ => bail!("Invalid diagnosis number {}", v),
        }
    }
}

impl fmt::Display for Diagnosis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Diagnosis::UNKNOWN => write!(f, "UNKNOWN"),
            Diagnosis::HL => write!(f, "HL"),
            Diagnosis::DLBCL => write!(f, "DLBCL"),
            Diagnosis::CLL => write!(f, "CLL"),
            Diagnosis::FL => write!(f, "FL"),
            Diagnosis::MCL => write!(f, "MCL"),
            Diagnosis::LTDS => write!(f, "LTDS"),
        }
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
