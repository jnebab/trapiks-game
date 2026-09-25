use std::collections::BTreeMap;

use serde::Deserialize;

pub type Tags = BTreeMap<String, String>;

#[derive(Clone, Debug, Deserialize)]
pub struct Response {
    pub elements: Vec<Element>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Element {
    Node {
        id: i64,
        lat: f64,
        lon: f64,
        #[serde(default)]
        tags: Tags,
    },
    Way {
        id: i64,
        nodes: Vec<i64>,
        #[serde(default)]
        geometry: Option<Vec<LatLon>>,
        #[serde(default)]
        tags: Tags,
    },
    Relation {
        id: i64,
        members: Vec<Member>,
        #[serde(default)]
        tags: Tags,
    },
    #[serde(other)]
    Other,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct LatLon {
    pub lat: f64,
    pub lon: f64,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct Member {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(rename = "ref")]
    pub id: i64,
    pub role: String,
    #[serde(default)]
    pub geometry: Option<Vec<LatLon>>,
    #[serde(default)]
    pub lat: Option<f64>,
    #[serde(default)]
    pub lon: Option<f64>,
}
