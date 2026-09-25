use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoadClass {
    Motorway,
    MotorwayLink,
    Trunk,
    TrunkLink,
    Primary,
    PrimaryLink,
    Secondary,
    SecondaryLink,
    Tertiary,
    TertiaryLink,
    Unclassified,
    Road,
    Residential,
    LivingStreet,
    Service,
}

impl RoadClass {
    pub fn rank(self) -> u8 {
        match self {
            Self::Motorway => 14,
            Self::MotorwayLink => 13,
            Self::Trunk => 12,
            Self::TrunkLink => 11,
            Self::Primary => 10,
            Self::PrimaryLink => 9,
            Self::Secondary => 8,
            Self::SecondaryLink => 7,
            Self::Tertiary => 6,
            Self::TertiaryLink => 5,
            Self::Unclassified => 4,
            Self::Road | Self::Residential => 3,
            Self::LivingStreet => 2,
            Self::Service => 1,
        }
    }

    pub fn default_lanes(self) -> u8 {
        match self {
            Self::Motorway | Self::Trunk | Self::Primary => 2,
            _ => 1,
        }
    }

    pub fn default_speed_kph(self) -> u8 {
        match self {
            Self::Motorway => 100,
            Self::MotorwayLink | Self::Trunk | Self::Primary => 60,
            Self::Secondary => 50,
            Self::TrunkLink | Self::PrimaryLink | Self::SecondaryLink | Self::Tertiary => 40,
            Self::TertiaryLink | Self::Unclassified | Self::Road => 30,
            Self::Residential => 20,
            Self::LivingStreet => 10,
            Self::Service => 15,
        }
    }

    pub fn implies_oneway(self) -> bool {
        matches!(self, Self::Motorway)
    }

    pub fn from_highway(value: &str) -> Option<RoadClass> {
        match value {
            "motorway" => Some(Self::Motorway),
            "motorway_link" => Some(Self::MotorwayLink),
            "trunk" => Some(Self::Trunk),
            "trunk_link" => Some(Self::TrunkLink),
            "primary" => Some(Self::Primary),
            "primary_link" => Some(Self::PrimaryLink),
            "secondary" => Some(Self::Secondary),
            "secondary_link" => Some(Self::SecondaryLink),
            "tertiary" => Some(Self::Tertiary),
            "tertiary_link" => Some(Self::TertiaryLink),
            "unclassified" => Some(Self::Unclassified),
            "road" => Some(Self::Road),
            "residential" => Some(Self::Residential),
            "living_street" => Some(Self::LivingStreet),
            "service" => Some(Self::Service),
            _ => None,
        }
    }
}
