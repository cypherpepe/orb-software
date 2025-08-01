use orb_mcu_interface::orb_messages;
use std::fmt::{Display, Formatter};

#[derive(Clone, Debug, Default)]
pub struct OrbRevision(pub orb_messages::Hardware);

trait OrbVersionFromInt {
    fn from_version_i32(value: i32) -> Self;
}
impl OrbVersionFromInt for orb_messages::hardware::OrbVersion {
    fn from_version_i32(value: i32) -> Self {
        match value {
            1 => orb_messages::hardware::OrbVersion::HwVersionPearlEv1,
            2 => orb_messages::hardware::OrbVersion::HwVersionPearlEv2,
            3 => orb_messages::hardware::OrbVersion::HwVersionPearlEv3,
            4 => orb_messages::hardware::OrbVersion::HwVersionPearlEv4,
            5 => orb_messages::hardware::OrbVersion::HwVersionPearlEv5,
            20 => orb_messages::hardware::OrbVersion::HwVersionDiamondPoc1,
            21 => orb_messages::hardware::OrbVersion::HwVersionDiamondPoc2,
            22 => orb_messages::hardware::OrbVersion::HwVersionDiamondB3,
            23 => orb_messages::hardware::OrbVersion::HwVersionDiamondEvt,
            24 => orb_messages::hardware::OrbVersion::HwVersionDiamondV44,
            25 => orb_messages::hardware::OrbVersion::HwVersionDiamondV45,
            26 => orb_messages::hardware::OrbVersion::HwVersionDiamondV46,
            _ => orb_messages::hardware::OrbVersion::HwVersionUnknown,
        }
    }
}

impl Display for OrbRevision {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match orb_messages::hardware::OrbVersion::from_version_i32(self.0.version) {
            orb_messages::hardware::OrbVersion::HwVersionUnknown => {
                // from_version_i32 returned unknown but the version might not be 0
                // meaning it's known by the firmware but not by the current binary
                if self.0.version
                    == orb_messages::hardware::OrbVersion::HwVersionUnknown as i32
                {
                    return write!(f, "unknown");
                }

                tracing::error!(
                    "A new orb revision might not be implemented by that binary: {:?}.",
                    self.0.version
                );

                // let's write if it's a pearl or diamond orb, guessing by the version number
                if self.0.version
                    < orb_messages::hardware::OrbVersion::HwVersionDiamondPoc1 as i32
                {
                    write!(f, "Pearl_unknown")
                } else {
                    write!(f, "Diamond_unknown")
                }
            }
            orb_messages::hardware::OrbVersion::HwVersionPearlEv1
            | orb_messages::hardware::OrbVersion::HwVersionPearlEv2
            | orb_messages::hardware::OrbVersion::HwVersionPearlEv3
            | orb_messages::hardware::OrbVersion::HwVersionPearlEv4
            | orb_messages::hardware::OrbVersion::HwVersionPearlEv5 => {
                write!(f, "EVT{:?}", self.0.version)
            }
            orb_messages::hardware::OrbVersion::HwVersionDiamondPoc1
            | orb_messages::hardware::OrbVersion::HwVersionDiamondPoc2
            | orb_messages::hardware::OrbVersion::HwVersionDiamondB3 => {
                write!(
                    f,
                    "Diamond_B{:?}",
                    self.0.version
                        - orb_messages::hardware::OrbVersion::HwVersionDiamondPoc1
                            as i32
                        + 1
                )
            }
            orb_messages::hardware::OrbVersion::HwVersionDiamondEvt
            | orb_messages::hardware::OrbVersion::HwVersionDiamondV44 => {
                write!(f, "Diamond_EVT")
            }
            orb_messages::hardware::OrbVersion::HwVersionDiamondV45 => {
                write!(f, "Diamond_DVT1")
            }
            orb_messages::hardware::OrbVersion::HwVersionDiamondV46 => {
                write!(f, "Diamond_DVT2")
            }
        }
    }
}
