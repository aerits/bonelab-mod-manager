use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manifest {
    pub version: u64,
    pub root: Root,
    pub objects: Object,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Root {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
// #[serde(untagged)]
pub struct Object {
    #[serde(rename = "1")]
    pub pallet: Pallet,
    #[serde(rename = "2")]
    pub mod_listing: Option<ModListing>,
    #[serde(rename = "3")]
    pub mod_target: Option<ModTarget>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Pallet {
    pub palletBarcode: String,
    pub palletPath: String,
    pub catalogPath: String,
    pub version: Option<String>,
    pub installedDate: String,
    pub updateDate: String,
    pub modListing: Option<Reference>,
    pub active: bool,
    pub isa: Isa,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModListing {
    pub barcode: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub author: Option<String>,
    pub version: Option<String>,
    pub thumbnailUrl: Option<String>,
    pub targets: HashMap<String, Reference>,
    pub isa: Isa,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ModTarget {
    pub thumbnailOverride: Option<String>,
    pub gameId: u64,
    pub modId: u64,
    pub modfileId: u64,
    pub isa: Isa,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Reference {
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(rename = "type")]
    pub type_: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Isa {
    #[serde(rename = "type")]
    pub type_: String,
}

// // Custom deserialization for Object
// impl<'de> Deserialize<'de> for Object {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // Deserialize into a serde_json::Value first
//         let value: Value = Value::deserialize(deserializer)?;

//         // Check for the presence of specific fields to determine the type
//         if let Some(pallet) = value.get("palletBarcode") {
//             let pallet: Pallet = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
//             return Ok(Object::Pallet(pallet));
//         } else if let Some(mod_listing) = value.get("barcode") {
//             let mod_listing: ModListing = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
//             return Ok(Object::ModListing(mod_listing));
//         } else if let Some(mod_target) = value.get("gameId") {
//             let mod_target: ModTarget = serde_json::from_value(value).map_err(serde::de::Error::custom)?;
//             return Ok(Object::ModTarget(mod_target));
//         }

//         Err(serde::de::Error::custom("Unknown object type"))
//     }
// }