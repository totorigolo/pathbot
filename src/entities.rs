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
            NW => 45.,
            W => 90.,
            SW => 135.,
            S => 180.,
            SE => 225.,
            E => 270.,
            NE => 315.,
        }
    }
}
