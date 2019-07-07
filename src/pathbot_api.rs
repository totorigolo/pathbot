//! Pathbot data (de)serialization
//!
//! See docs/API.md.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::From;

use crate::entities::*;

/*
 * RawRoom
 */

/// Needed because of maze_exit_hint which is one struct in Room
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
pub(crate) struct RawRoom {
    status: RoomStatus,
    message: String,
    exits: Vec<MoveDirection>,
    description: String,
    mazeExitDirection: CompassDirection,
    mazeExitDistance: u32,
    locationPath: String,
}

impl From<RawRoom> for Room {
    fn from(raw_room: RawRoom) -> Self {
        Room {
            status: raw_room.status,
            message: raw_room.message,
            exits: raw_room.exits,
            description: raw_room.description,
            maze_exit_hint: MazeExitHint {
                direction: raw_room.mazeExitDirection,
                distance: raw_room.mazeExitDistance,
            },
            location_path: raw_room.locationPath,
        }
    }
}

/*
 * RoomStatus
 */

impl Serialize for RoomStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use RoomStatus::*;
        serializer.serialize_str(match self {
            InProgress => "in-progress",
            Finished => "finished",
        })
    }
}

impl<'de> Deserialize<'de> for RoomStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use RoomStatus::*;
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "in-progress" => InProgress,
            "finished" => Finished,
            _ => return Err(serde::de::Error::custom(format!("Unknown status: {}", s))),
        })
    }
}

/*
 * MoveDirection
 */

impl Serialize for MoveDirection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.short_name())
    }
}

impl<'de> Deserialize<'de> for MoveDirection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use MoveDirection::*;
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "N" => N,
            "S" => S,
            "E" => E,
            "W" => W,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Unknown move direction: {}",
                    s
                )))
            }
        })
    }
}

/*
 * CompassDirection
 */

impl Serialize for CompassDirection {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.short_name())
    }
}

impl<'de> Deserialize<'de> for CompassDirection {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use CompassDirection::*;
        let s = String::deserialize(deserializer)?;
        Ok(match s.as_str() {
            "N" => N,
            "S" => S,
            "E" => E,
            "W" => W,
            "NW" => NW,
            "NE" => NE,
            "SW" => SW,
            "SE" => SE,
            _ => {
                return Err(serde::de::Error::custom(format!(
                    "Unknown compass direction: {}",
                    s
                )))
            }
        })
    }
}
