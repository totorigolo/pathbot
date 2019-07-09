#![recursion_limit = "256"]

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

type NotificationId = u32;
type RoomId = String;

#[derive(PartialEq, Debug, Clone)]
pub struct State {
    rooms: HashMap<RoomId, Room>,
    status: Status,
}

#[derive(PartialEq, Debug, Clone)]
pub enum Status {
    Loading,
    InRoom(RoomId),
    /// We store the received exit message.
    Finished(Exit),
}

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

pub enum Msg {
    Init,
    FetchNextRoom(MoveDirection),
    Fetching,
    ReceivedRoom(Room),
    ReceivedMessage(Message),
    ReceivedExit(Exit),
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
            status: Status::Loading,
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
            Msg::FetchNextRoom(direction) => match self.state.status.clone() {
                Status::Loading => error!("Logic error: no current room."),
                Status::InRoom(current_room_id) => {
                    self.fetch(FetchRoomRequest::NextRoom(current_room_id, direction));
                }
                Status::Finished(_) => error!("Logic error: no more room."),
            },
            Msg::Fetching => {
                return false;
            }
            Msg::ReceivedRoom(room) => {
                self.fetching = false;
                self.state.status = Status::InRoom(room.location_path.clone());
                self.state.rooms.insert(room.location_path.clone(), room);
            }
            Msg::ReceivedMessage(message) => {
                self.fetching = false;

                self.link.send_self(Msg::NewNotification(Notification {
                    message: format!("{}", message.message),
                    level: NotificationLevel::Warning,
                }));
            }
            Msg::ReceivedExit(exit) => {
                self.fetching = false;
                self.state.status = Status::Finished(exit);

                self.link.send_self(Msg::NewNotification(Notification {
                    message: "Congratulations! You exited the maze!".to_string(),
                    level: NotificationLevel::Success,
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
        self.fetching || self.state.status == Status::Loading
    }

    fn current_room(&self) -> Option<&Room> {
        match &self.state.status {
            Status::InRoom(id) => self.state.rooms.get(id),
            _ => None,
        }
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
        let status_to_str = |status| match status {
            RoomStatus::InProgress => "In progress",
            RoomStatus::Finished => "Finished",
        };
        let exit_li = |direction: &MoveDirection| html! {
            <li>{ direction.long_name() }</li>
        };
        match &self.state.status {
            Status::Loading => html! { <h1>{ "Loading..." }</h1> },
            Status::InRoom(room_id) => {
                if let Some(room) = self.state.rooms.get(room_id) {
                    html! {
                        <div>
                            <p id="status">{ status_to_str(room.status) }</p>
                            <p id="message">{ &room.message }</p>
                            <p id="description">{ &room.description }</p>
                            <p id="exits">
                                { "This room has " }
                                { format!("{}", room.exits.len()) }
                                { " exit" }
                                { if room.exits.len() == 1 { "" } else { "s" } }
                                { ": " }
                                <ul>
                                    { for room.exits.iter().map(exit_li) }
                                </ul>
                            </p>
                            <p id="question-action">
                                { "What do you want to do?" }
                            </p>
                        </div>
                    }
                } else {
                    html! {
                        <p>{ "Error: unknown room." }</p>
                    }
                }
            },
            Status::Finished(exit) => html! {
                <div>
                    <p id="status">{ status_to_str(exit.status) }</p>
                    <p id="description">{ &exit.description }</p>
                </div>
            },
        }
    }

    fn view_buttons(&self) -> Html<Model> {
        if !self.loading() {
            use MoveDirection::*;
            let button = |direction: MoveDirection| html! {
                <button class="btn btn--primary" style="margin-left: 5px;"
                    onclick=|_| Msg::FetchNextRoom(direction)>
                    { "Go " }{ direction.long_name() }
                </button>
            };
            html! {
                <div id="buttons">
                    { for [W, N, S, E].iter().cloned().map(button) }
                    <button class="btn btn--primary" style="margin-left: 5px;"
                        onclick=|_| Msg::Init>
                        { "Restart" }
                    </button>
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
                        Ok(PathbotApiMessage::Exit(exit)) =>
                            Msg::ReceivedExit(exit),
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
