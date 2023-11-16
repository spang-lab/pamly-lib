use anyhow::{bail, Result};
use pyo3::{pyclass, pymethods, PyResult};
use std::convert::TryFrom;
use strum::{Display, EnumIter, EnumString, IntoEnumIterator};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIter, Display, EnumString)]
#[repr(u8)]
#[pyclass]
#[rustfmt::skip]
#[strum(ascii_case_insensitive)]
pub enum TileLabel {
    Unlabeled                = 0x00,
    Unknown                  = 0x01,
    Other                    = 0x02,
    NonExisting              = 0x03,
    Artefact                 = 0x04,
    Empty                    = 0x05,
    Tumor                    = 0x10,
    TumorPartial             = 0x11,
    TumorInvasive            = 0x12,
    ConnectiveTissue         = 0x13,
    Blood                    = 0x14,
    BloodVessel              = 0x15,
    FattyTissue              = 0x16,
    Necrosis                 = 0x17,
    Lymphatic                = 0x18,
    Muscle                   = 0x19,
    MuscleSmooth             = 0x1A,
    MuscleStriated           = 0x1B,
    Mucosa                   = 0x1C,
    MucosaStomach            = 0x1D,
    MucosaLargeIntestine     = 0x1E,
    MucosaSmallIntestine     = 0x1F,
    Epithelium               = 0x20,
    EpitheliumSquamous       = 0x21,
    EpitheliumGland          = 0x22,
    Cns                      = 0x23,
    Bone                     = 0x24,
    Bonemarrow               = 0x25,
}
impl TileLabel {
    pub fn from(s: &str) -> Result<TileLabel> {
        let clean = s.replace(" ", "").replace("_", "");
        match TileLabel::try_from(clean.as_str()) {
            Ok(l) => Ok(l),
            Err(e) => bail!(e.to_string()),
        }
    }
}

#[pymethods]
impl TileLabel {
    #[staticmethod]
    pub fn list() -> Vec<TileLabel> {
        TileLabel::iter().collect()
    }
    #[new]
    pub fn new(s: &str) -> PyResult<TileLabel> {
        let label = TileLabel::from(s)?;
        Ok(label)
    }
    pub fn to_string(&self) -> String {
        let s = format!("{}", self);
        s
    }
}

impl TryFrom<u8> for TileLabel {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<TileLabel> {
        let labels = TileLabel::list();
        for label in labels {
            if label as u8 == value {
                return Ok(label);
            }
        }
        bail!("No label for number {}", value);
    }
}

#[cfg(test)]
mod tests {
    use super::{Result, TileLabel, TryFrom};
    #[test]
    fn label_consistency() -> Result<()> {
        for stain in TileLabel::list() {
            let num = stain as u8;
            let stain2 = TileLabel::try_from(num)?;
            assert_eq!(stain, stain2);
        }
        Ok(())
    }
    #[test]
    fn parsing() -> Result<()> {
        let unlabeled = TileLabel::from("Unlabeled")?;
        assert_eq!(unlabeled, TileLabel::Unlabeled);
        Ok(())
    }
    #[test]
    fn parsing_lowercase() -> Result<()> {
        let unlabeled = TileLabel::from("unlabeled")?;
        assert_eq!(unlabeled, TileLabel::Unlabeled);
        Ok(())
    }
    #[test]
    fn parsing_with_underscore() -> Result<()> {
        let epi = TileLabel::from("Epithelium_Squamous")?;
        assert_eq!(epi, TileLabel::EpitheliumSquamous);
        Ok(())
    }
    #[test]
    fn parsing_with_spaces() -> Result<()> {
        let epi = TileLabel::from("Epithelium    Squamous")?;
        assert_eq!(epi, TileLabel::EpitheliumSquamous);
        Ok(())
    }
    #[test]
    fn parsing_err() -> Result<()> {
        let err = TileLabel::from("Invalid Label");
        assert!(err.is_err());
        Ok(())
    }
}
