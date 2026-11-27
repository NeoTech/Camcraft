//! Provenance: explicit cutter handedness and a safe refusal, from
//! issue #4031 (the law the tap hand set in #1453, on the spindle word).
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CutterHand {
    #[default]
    Right,
    Left,
}

impl CutterHand {
    pub fn is_right(&self) -> bool {
        *self == Self::Right
    }
}

impl<'de> Deserialize<'de> for CutterHand {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match serde_json::Value::deserialize(deserializer)?.as_str() {
            Some("right") => Ok(Self::Right),
            Some("left") => Ok(Self::Left),
            _ => Err(serde::de::Error::custom("cutter hand must be right or left")),
        }
    }
}
