#![feature(option_flattening)]

use log::*;
use failure::{Error, bail, format_err};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryInto;
use yew::prelude::*;
use yew::html;
use yew::format::{Json, Nothing};
use yew::services::{
    fetch::{FetchService, FetchTask, Request, Response},
};

mod components;
use components::compass::Compass;

pub struct Model {
    state: State,
    link: ComponentLink<Model>,
    fetch_service: FetchService,
    fetching: bool,
    fetch_task: Option<FetchTask>,
}

pub struct State {
    rooms: HashMap<String, Room>,
    current_room_id: Option<String>,
}

/// Pathbot data format
///
/// Notes:
/// - status: either "in-progress" or "finished"
/// - exits entries: one of N, S, E, W
/// - mazeExitDirection: one of N, S, E, W, NW, NE, SW, SE
///
/// FIXME: This will panic for the last room because of missing fields
///
#[derive(Serialize, Deserialize, Debug)]
#[allow(non_snake_case)]
struct RawRoom {
    status: String,
    message: String,
    exits: Vec<String>,
    description: String,
    mazeExitDirection: String,
    mazeExitDistance: u32,
    locationPath: String,
}

#[derive(Clone, Debug)]
pub struct Room {
    status: String,
    message: String,
    exits: Vec<String>,
    description: String,
    maze_exit_hint: MazeExitHint,
    location_path: String,
}

#[derive(PartialEq, Debug, Copy, Clone)]
pub struct MazeExitHint {
    direction: CompassDirection,
    distance: u32,
}

impl TryInto<Room> for RawRoom {
    type Error = Error;
    fn try_into(self) -> Result<Room, Error> {
        use CompassDirection::*;
        let direction = match self.mazeExitDirection.as_ref() {
            "N" => N,
            "S" => S,
            "E" => E,
            "W" => W,
            "NW" => NW,
            "NE" => NE,
            "SW" => SW,
            "SE" => SE,
            _ => bail!("Unknown direction: {}", self.mazeExitDirection),
        };
        Ok(Room {
            status: self.status,
            message: self.message,
            exits: self.exits,
            description: self.description,
            maze_exit_hint: MazeExitHint {
                direction,
                distance: self.mazeExitDistance,
            },
            location_path: self.locationPath,
        })
    }
}

// TODO: Use this in Room
#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum MoveDirection {
    N,
    S,
    E,
    W,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct Move {
    direction: MoveDirection,
}

pub enum Msg {
    Init,
    FetchNextRoom(MoveDirection),
    ReceivedRoom(Room),
    FetchFailed(Error),
    ButtonPressed,
    Nope,
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq)]
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

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = State {
            rooms: HashMap::default(),
            current_room_id: None,
        };
        Model {
            state,
            link,
            fetch_service: FetchService::new(),
            fetching: false,
            fetch_task: None,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Init => {
                // Fetch the start room
                let request = Request::post("https://api.noopschallenge.com/pathbot/start")
                    .body(Nothing)
                    .expect("Failed to build request.");
                self.fetch(request);
            }
            Msg::FetchNextRoom(direction) => {
                if let Some(current_room_id) = &self.state.current_room_id {
                    let url = format!(
                        "https://api.noopschallenge.com{}",
                        current_room_id
                    );
                    let body = Move { direction };
                    let request = Request::post(url)
                        .header("Content-Type", "application/json")
                        .body(Json(&body))
                        .expect("Failed to build request.");
                    self.fetch(request);
                } else {
                    error!("Logic error: no current room.");
                }
            }
            Msg::ReceivedRoom(room) => {
                self.fetching = false;
                info!("Received new room: {:?}", room);
                self.state.current_room_id = Some(room.location_path.clone());
                self.state.rooms.insert(room.location_path.clone(), room);
            }
            Msg::FetchFailed(response) => {
                self.fetching = false;
                error!("Fetching failed: {:?}", response);
            }
            Msg::ButtonPressed => {
                info!("Button pressed");
            }
            Msg::Nope => {
                return false;
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let current_room = self.current_room();
        let exit_hint = current_room.map(|r| r.maze_exit_hint.clone());
        html! {
            <div class="pathbot-wrapper",>
                <section id="main",>
                    { self.view_room() }
                    { self.view_buttons() }
                    <Compass: maze_exit_hint=exit_hint,/>
                </section>
            </div>
        }
    }
}

impl Model {
    fn loading(&self) -> bool {
        self.fetching || self.state.current_room_id.is_none()
    }

    fn current_room(&self) -> Option<&Room> {
        self.state.current_room_id
            .as_ref()
            .map(|id| self.state.rooms.get(id))
            .flatten()
    }

    fn view_room(&self) -> Html<Model> {
        if let Some(room_id) = &self.state.current_room_id {
            if let Some(room) = self.state.rooms.get(room_id) {
                html!{
                    <p id="status",>{ &room.status }</p>
                    <p id="message",>{ &room.message }</p>
                    <p id="exits",>{ format!("{:?}", room.exits) }</p>
                    <p id="description",>{ &room.description }</p>
                }
            } else {
                html!{
                    <p>{ "Error: unknown room." }</p>
                }
            }
        } else {
            html! {
                <h1>{ "Loading..." }</h1>
            }
        }
    }

    fn view_buttons(&self) -> Html<Model> {
        if !self.loading() {
            html!{
                <div id="buttons",>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::W),>{ "W" }</button>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::N),>{ "N" }</button>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::S),>{ "S" }</button>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::E),>{ "E" }</button>
                </div>
            }
        } else {
            html!{
                <p>{ "Please wait" }</p>
            }
        }
    }

    fn fetch<IN: Into<yew::format::Text>>(&mut self, request: Request<IN>) {
        if self.fetching {
            warn!("Not sending, ongoing request.");
            return;
        }
        self.fetching = true;

        // Send the request
        let callback = self
            .link
            .send_back(move |response: Response<Json<Result<RawRoom, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    match data.and_then(|raw| raw.try_into()) {
                        Ok(room) => Msg::ReceivedRoom(room),
                        Err(e) => Msg::FetchFailed(e),
                    }
                } else {
                    match data {
                        Ok(received) => Msg::FetchFailed(format_err!("Received error: {:?}", received)),
                        Err(e) => Msg::FetchFailed(e)
                    }
                }
            });
        let task = self.fetch_service.fetch(request, callback);
        self.fetch_task = Some(task);
    }
}

// impl State {
//     fn n_rooms(&self) -> usize {
//         self.rooms.len()
//     }
// }
