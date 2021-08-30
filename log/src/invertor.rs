use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InvertorParametr {
    #[serde(deserialize_with = "adr_from_str")]
    #[serde(serialize_with = "adr_to_str")]
    pub address: (u8, u8),
    pub value: u32,
    pub name: String,
} 

impl InvertorParametr {
    pub fn address(&self) -> u16 {
        self.address.0 as u16 * 256 
        + self.address.1 as u16
    }
    pub fn parametr(address: u16) -> (u8, u8) {
        (
            (address / 256) as u8,
            (address % 256) as u8
        )
    }
    pub fn parametr_str(address: u16) -> String {
        let p = Self::parametr(address);
        format!("({}, {})", p.0, p.1)
    }
}

use serde::{de, de::Error, Deserializer, Serializer};
pub(crate) fn adr_from_str<'de, D>(deserializer: D) -> Result<(u8, u8), D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;
    let adr = s.trim_matches(|c| c == '(' || c==')').split_once(',').ok_or("Address Invalid").map_err(D::Error::custom)?;
    Ok((
        adr.0.trim().parse().map_err(D::Error::custom)?,
        adr.1.trim().parse().map_err(D::Error::custom)?
    ))
}

pub(crate) fn adr_to_str<S>(adr: &(u8, u8), serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
//     let s = dt.to_rfc3339_opts(SecondsFormat::Millis, false);
    let s = format!("({}, {})", adr.0, adr.1);
    serializer.serialize_str(&s)
}
