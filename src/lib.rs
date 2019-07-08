#![recursion_limit = "128"]

use failure::{format_err, Error};
use log::*;
use serde_json::json;
use std::collections::HashMap;
use yew::format::Json;
use yew::html;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};

mod components;
mod entities;
mod pathbot_api;

use entities::*;

pub struct Model {
    state: State,
    link: ComponentLink<Model>,
    fetch_service: FetchService,
    fetching: bool,
    fetch_task: Option<FetchTask>,
    notifications: HashMap<NotificationId, Notification>,
    next_notification_id: NotificationId,
}

pub type NotificationId = u32;

#[derive(PartialEq, Debug, Clone)]
pub struct Notification {
    message: String,
    level: NotificationLevel,
}

#[derive(PartialEq, Debug, Clone)]
pub enum NotificationLevel {
    Info,
    Success,
    Warning,
    Danger,
}

pub struct State {
    rooms: HashMap<String, Room>,
    current_room_id: Option<String>,
}

pub enum Msg {
    Init,
    FetchNextRoom(MoveDirection),
    ReceivedRoom(Room),
    FetchRoomFailed(Error),
    ButtonPressed,
    NewNotification(Notification),
    NotificationClosed(NotificationId),
}

enum FetchRoomRequest {
    StartRoom,
    NextRoom(LocationPath, MoveDirection),
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
            notifications: HashMap::default(),
            next_notification_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Init => {
                self.fetch(FetchRoomRequest::StartRoom);
            }
            Msg::FetchNextRoom(direction) => match self.state.current_room_id.clone() {
                Some(current_room_id) => {
                    self.fetch(FetchRoomRequest::NextRoom(current_room_id, direction));
                }
                None => error!("Logic error: no current room."),
            },
            Msg::ReceivedRoom(room) => {
                self.fetching = false;
                self.state.current_room_id = Some(room.location_path.clone());
                self.state.rooms.insert(room.location_path.clone(), room);

                self.link.send_self(Msg::NewNotification(Notification {
                    message: "Received a new room.".to_string(),
                    level: NotificationLevel::Info,
                }));
            }
            Msg::FetchRoomFailed(response) => {
                self.fetching = false;
                error!("Fetching room failed: {:?}", response);
            }
            Msg::ButtonPressed => {
                info!("Button pressed");
            }
            Msg::NewNotification(notification) => {
                let id = self.next_notification_id;
                self.next_notification_id += 1;

                self.notifications.insert(id, notification);
            }
            Msg::NotificationClosed(notification_id) => {
                self.notifications.remove(&notification_id);
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let current_room = self.current_room();
        let exit_hint = current_room.map(|r| r.maze_exit_hint);
        html! {
            <div class="pathbot-wrapper",>
                <section id="main",>
                    { self.view_notifications() }
                    { self.view_room() }
                    { self.view_buttons() }
                    <components::Compass: maze_exit_hint=exit_hint,/>
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
        self.state
            .current_room_id
            .as_ref()
            .and_then(|id| self.state.rooms.get(id))
    }
}

impl Model {
    fn view_notifications(&self) -> Html<Model> {
        html! {
            <div id="notifications",>
                { for self.notifications
                        .iter()
                        .map(|(id, notif)| view_notification(id, notif)) }
            </div>
        }
    }

    fn view_room(&self) -> Html<Model> {
        if let Some(room_id) = &self.state.current_room_id {
            if let Some(room) = self.state.rooms.get(room_id) {
                let status = match room.status {
                    RoomStatus::InProgress => "In progress",
                    RoomStatus::Finished => "Finished",
                };
                html! {
                    <div>
                        <p id="status",>{ status }</p>
                        <p id="message",>{ &room.message }</p>
                        <p id="exits",>{ format!("{:?}", room.exits) }</p>
                        <p id="description",>{ &room.description }</p>
                    </div>
                }
            } else {
                html! {
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
            html! {
                <div id="buttons",>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::W),>{ "W" }</button>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::N),>{ "N" }</button>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::S),>{ "S" }</button>
                    <button class="", onclick=|_| Msg::FetchNextRoom(MoveDirection::E),>{ "E" }</button>
                </div>
            }
        } else {
            html! {
                <p>{ "Please wait" }</p>
            }
        }
    }
}

fn view_notification(id: &NotificationId, notification: &Notification) -> Html<Model> {
    let id = id.clone();
    html! {
        <components::Notification: notification=notification.clone(),
            on_close=move |_| Msg::NotificationClosed(id),/>
    }
}

impl Model {
    fn fetch(&mut self, request: FetchRoomRequest) {
        if self.fetching {
            warn!("Not sending, ongoing request.");
            return;
        }
        self.fetching = true;

        self.link.send_self(Msg::NewNotification(Notification {
            message: "Fetching...".to_string(),
            level: NotificationLevel::Info,
        }));

        // Build the request
        let request: Request<yew::format::Text> = match request {
            FetchRoomRequest::StartRoom => {
                Request::post("https://api.noopschallenge.com/pathbot/start")
                    .header("Content-Type", "application/json")
                    .body(Ok("".to_string()))
                    .unwrap() // cannot really fail (except OOM)
            }
            FetchRoomRequest::NextRoom(location_path, move_direction) => {
                let url = format!("https://api.noopschallenge.com{}", location_path);
                let body = json!({ "direction": move_direction });
                Request::post(url)
                    .header("Content-Type", "application/json")
                    .body(Json(&body).into())
                    .unwrap() // cannot really fail (except OOM)
            }
        };

        // Send the request
        use pathbot_api::RawRoom;
        let callback =
            self.link
                .send_back(move |response: Response<Json<Result<RawRoom, Error>>>| {
                    let (meta, Json(data)) = response.into_parts();
                    if meta.status.is_success() {
                        match data.map(|raw| raw.into()) {
                            Ok(room) => Msg::ReceivedRoom(room),
                            Err(e) => Msg::FetchRoomFailed(e),
                        }
                    } else {
                        match data {
                            Ok(received) => {
                                Msg::FetchRoomFailed(format_err!("Received error: {:?}", received))
                            }
                            Err(e) => Msg::FetchRoomFailed(e),
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
