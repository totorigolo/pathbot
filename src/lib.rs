use log::*;
use failure::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use yew::prelude::*;
use yew::html;
use yew::format::{Json, Nothing};
use yew::services::{
    fetch::{FetchService, FetchTask, Request, Response},
};

pub struct Model {
    state: State,
    link: ComponentLink<Model>,
    fetch_service: FetchService,
    fetching: bool,
    fetch_task: Option<FetchTask>,
}

#[derive(Serialize, Deserialize)]
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
pub struct Room {
    status: String,
    message: String,
    exits: Vec<String>,
    description: String,
    mazeExitDirection: String,
    mazeExitDistance: u32,
    locationPath: String,
}

// TODO: Use this in Room
#[derive(Serialize, Deserialize, Debug)]
pub enum MoveDirection {
    N,
    S,
    E,
    W,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Move {
    direction: MoveDirection,
}

pub enum Msg {
    Init,
    FetchNextRoom(MoveDirection),
    FetchReady(Result<Room, Error>),
    FetchFailed(Result<Room, Error>),
    ButtonPressed,
    Nope,
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
            Msg::FetchReady(response) => {
                self.fetching = false;
                match response {
                    Ok(room) => {
                        info!("Received new room: {:?}", room);
                        self.state.current_room_id = Some(room.locationPath.clone());
                        self.state.rooms.insert(room.locationPath.clone(), room);
                    },
                    Err(e) => error!("Receive room failed: {:?}", e),
                }
            }
            Msg::FetchFailed(response) => {
                self.fetching = false;
                error!("Failed: {:?}", response);
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
        html! {
            <div class="pathbot-wrapper",>
                <section id="main",>
                    { self.view_room() }
                    { self.view_buttons() }
                </section>
            </div>
        }
    }
}

impl Model {
    fn loading(&self) -> bool {
        self.fetching || self.state.current_room_id.is_none()
    }

    fn view_room(&self) -> Html<Model> {
        if let Some(room_id) = &self.state.current_room_id {
            if let Some(room) = self.state.rooms.get(room_id) {
                html!{
                    <p id="status",>{ &room.status }</p>
                    <p id="message",>{ &room.message }</p>
                    <p id="exits",>{ format!("{:?}", room.exits) }</p>
                    <p id="description",>{ &room.description }</p>
                    <p id="mazeExitDirection",>{ &room.mazeExitDirection }</p>
                    <p id="mazeExitDistance",>{ room.mazeExitDistance }</p>
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
            .send_back(move |response: Response<Json<Result<Room, Error>>>| {
                let (meta, Json(data)) = response.into_parts();
                if meta.status.is_success() {
                    Msg::FetchReady(data)
                } else {
                    Msg::FetchFailed(data)
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
