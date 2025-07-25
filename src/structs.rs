use serde::{Deserialize, Serialize};
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

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
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

#[allow(non_snake_case)]
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
