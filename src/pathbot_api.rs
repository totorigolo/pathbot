//! Pathbot data (de)serialization
//!
//! See docs/API.md.
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::convert::From;

/*
 * Public structs
 */

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(untagged)]
pub enum PathbotApiMessage {
    Room(Room),
    Message(Message),
    Exit(Exit),
}

#[derive(PartialEq, Debug, Clone)]
pub struct Room {
    pub status: RoomStatus,
    pub message: String,
    pub exits: Vec<MoveDirection>,
    pub description: String,
    pub maze_exit_hint: MazeExitHint,
    pub location_path: LocationPath,
}

pub type LocationPath = String;

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum RoomStatus {
    InProgress,
    Finished,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct MazeExitHint {
    pub direction: CompassDirection,
    pub distance: u32,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum MoveDirection {
    N,
    S,
    E,
    W,
}

impl MoveDirection {
    pub fn short_name(self) -> &'static str {
        use MoveDirection::*;
        match self {
            N => "N",
            S => "S",
            E => "E",
            W => "W",
        }
    }

    pub fn long_name(self) -> &'static str {
        use MoveDirection::*;
        match self {
            N => "North",
            S => "South",
            E => "East",
            W => "West",
        }
    }

    /// Returns clockwise angle
    pub fn angle_deg(self) -> f32 {
        use MoveDirection::*;
        match self {
            N => 0.,
            E => 90.,
            S => 180.,
            W => 270.,
        }
    }
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub enum CompassDirection {
    N,
    S,
    E,
    W,
    NW,
    NE,
    SW,
    SE,
}

impl CompassDirection {
    pub fn short_name(self) -> &'static str {
        use CompassDirection::*;
        match self {
            N => "N",
            S => "S",
            E => "E",
            W => "W",
            NW => "NW",
            NE => "NE",
            SW => "SW",
            SE => "SE",
        }
    }

    pub fn long_name(self) -> &'static str {
        use CompassDirection::*;
        match self {
            N => "North",
            S => "South",
            E => "East",
            W => "West",
            NW => "North-West",
            NE => "North-East",
            SW => "South-West",
            SE => "South-East",
        }
    }

    /// Returns clockwise angle
    pub fn angle_deg(self) -> f32 {
        use CompassDirection::*;
        match self {
            N => 0.,
            NE => 45.,
            E => 90.,
            SE => 135.,
            S => 180.,
            SW => 225.,
            W => 270.,
            NW => 315.,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Message {
    pub message: String,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub struct Exit {
    pub status: RoomStatus,
    pub description: String,
}

/*
 * RawRoom
 */

/// This struct is needed because of maze_exit_hint which is one struct
/// in Room.
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct RawRoom {
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

impl From<Room> for RawRoom {
    fn from(room: Room) -> Self {
        RawRoom {
            status: room.status,
            message: room.message,
            exits: room.exits,
            description: room.description,
            mazeExitDirection: room.maze_exit_hint.direction,
            mazeExitDistance: room.maze_exit_hint.distance,
            locationPath: room.location_path,
        }
    }
}

/*
 * Room
 */

impl Serialize for Room {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let raw_room: RawRoom = self.clone().into();
        raw_room.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Room {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_room = RawRoom::deserialize(deserializer)?;
        Ok(raw_room.into())
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
