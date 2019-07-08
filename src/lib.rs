#![recursion_limit = "128"]

use failure::Error;
use log::*;
use serde_json::json;
use std::collections::HashMap;
use yew::format::Json;
use yew::html;
use yew::prelude::*;
use yew::services::fetch::{FetchService, FetchTask, Request, Response};
use linked_hash_map::LinkedHashMap;

mod components;
mod pathbot_api;

use pathbot_api::*;

pub struct Model {
    state: State,
    link: ComponentLink<Model>,
    fetch_service: FetchService,
    fetching: bool,
    fetch_task: Option<FetchTask>,
    notifications: LinkedHashMap<NotificationId, Notification>,
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
    Fetching,
    ReceivedRoom(Room),
    ReceivedMessage(Message),
    FetchRoomFailed(Error),
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
            notifications: LinkedHashMap::default(),
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
            Msg::Fetching => {
                return false;
            }
            Msg::ReceivedRoom(room) => {
                self.fetching = false;
                self.state.current_room_id = Some(room.location_path.clone());
                self.state.rooms.insert(room.location_path.clone(), room);

                self.link.send_self(Msg::NewNotification(Notification {
                    message: "Received a new room.".to_string(),
                    level: NotificationLevel::Info,
                }));
            }
            Msg::ReceivedMessage(message) => {
                self.fetching = false;

                self.link.send_self(Msg::NewNotification(Notification {
                    message: format!("{}", message.message),
                    level: NotificationLevel::Warning,
                }));
            }
            Msg::FetchRoomFailed(response) => {
                self.fetching = false;
                error!("Fetching room failed: {:?}", response);

                self.link.send_self(Msg::NewNotification(Notification {
                    message: format!(
                        "An error occurred while communicating \
                         with the API: {}",
                        response
                    ),
                    level: NotificationLevel::Warning,
                }));
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
            <section>
                { self.view_notifications() }
                <components::Compass: maze_exit_hint=exit_hint/>
                { self.view_room() }
                { self.view_buttons() }
            </section>
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
        let view_notification = |id: &NotificationId, notification: &Notification| {
            let id = id.clone();
            html! {
                <components::Notification: notification=notification.clone()
                    on_close=move |_| Msg::NotificationClosed(id)/>
            }
        };

        html! {
            <div id="notifications">
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
                        <p id="status">{ status }</p>
                        <p id="message">{ &room.message }</p>
                        <p id="exits">{ format!("{:?}", room.exits) }</p>
                        <p id="description">{ &room.description }</p>
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
            use MoveDirection::*;
            html! {
                <div id="buttons">
                    <button class="btn btn--primary"
                        onclick=|_| Msg::FetchNextRoom(W)>{ "W" }</button>
                    <button class="btn btn--primary"
                        onclick=|_| Msg::FetchNextRoom(N)>{ "N" }</button>
                    <button class="btn btn--primary"
                        onclick=|_| Msg::FetchNextRoom(S)>{ "S" }</button>
                    <button class="btn btn--primary"
                        onclick=|_| Msg::FetchNextRoom(E)>{ "E" }</button>
                </div>
            }
        } else {
            html! {
                <p>{ "Please wait" }</p>
            }
        }
    }
}

impl Model {
    fn fetch(&mut self, request: FetchRoomRequest) {
        if self.fetching {
            warn!("Not sending, ongoing request.");
            return;
        }
        self.fetching = true;
        self.link.send_self(Msg::Fetching);

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
        let callback =
            self.link
                .send_back(move |response: Response<Json<Result<_, Error>>>| {
                    let (_meta, Json(data)) = response.into_parts();
                    match data {
                        Ok(PathbotApiMessage::Room(room)) =>
                            Msg::ReceivedRoom(room),
                        Ok(PathbotApiMessage::Message(message)) =>
                            Msg::ReceivedMessage(message),
                        Err(e) => Msg::FetchRoomFailed(e),
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
