#![recursion_limit = "256"]

#[macro_use]
extern crate stdweb;

use failure::Error;
use linked_hash_map::LinkedHashMap;
use log::*;
use serde_json::json;
use std::{collections::HashMap, ops::Add};
use stdweb::{
    traits::*,
    unstable::TryInto,
    web::{
        document,
        html_element::CanvasElement,
        CanvasRenderingContext2d,
        event::KeyDownEvent,
    },
    traits::IKeyboardEvent,
};
use yew::{
    format::{Json, Text},
    html,
    prelude::*,
    services::{
        fetch::{FetchService, FetchTask, Request, Response},
    }
};

mod components;
mod pathbot_api;
mod services;

use pathbot_api::*;
use services::{KeydownService, KeydownTask};

pub struct Model {
    state: State,
    link: ComponentLink<Model>,

    fetch_service: FetchService,
    fetching: bool,
    fetching_move: Option<MoveDirection>,
    fetch_task: Option<FetchTask>,

    keydown_service: KeydownService,
    keydown_task: Option<KeydownTask>,

    /// This is a LinkedHashMap to enable iteration in insertion order.
    notifications: LinkedHashMap<NotificationId, Notification>,
    next_notification_id: NotificationId,
}

type NotificationId = u32;

#[derive(PartialEq, Debug, Clone)]
pub struct State {
    rooms: HashMap<RoomId, (Room, Coordinate)>,
    room_coords: HashMap<RoomId, Coordinate>,
    coord_to_id: HashMap<Coordinate, RoomId>,
    status: Status,
}

impl Default for State {
    fn default() -> Self {
        State {
            rooms: HashMap::default(),
            room_coords: HashMap::default(),
            coord_to_id: HashMap::default(),
            status: Status::Loading,
        }
    }
}

type RoomId = String;

#[derive(PartialEq, Eq, Hash, Debug, Copy, Clone)]
pub struct Coordinate {
    pub x: i32,
    pub y: i32,
}

impl Add for Coordinate {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Coordinate {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
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
    HandleKeyDown(KeyDownEvent),
    FetchNextRoom(MoveDirection),
    /// Contains the last move.
    ReceivedRoom(Room, Option<MoveDirection>),
    MoveToRoom(RoomId),
    ReceivedMessage(Message),
    /// Contains the last move.
    ReceivedExit(Exit, Option<MoveDirection>),
    FetchRoomFailed(Error),
    NewNotification(Notification),
    NotificationClosed(NotificationId),
    ClearNotifications,
    Noop,
}

enum FetchRoomRequest {
    StartRoom,
    NextRoom(LocationPath, MoveDirection),
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        let state = State::default();
        Model {
            state,
            link,

            fetch_service: FetchService::new(),
            fetching: false,
            fetching_move: None,
            fetch_task: None,

            keydown_service: KeydownService::new(),
            keydown_task: None,

            notifications: LinkedHashMap::default(),
            next_notification_id: 0,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Init => {
                self.state.restart();
                self.fetch(FetchRoomRequest::StartRoom);

                let cb = self.link.send_back(|e| Msg::HandleKeyDown(e));
                self.keydown_task = Some(self.keydown_service.spawn(cb));
            }
            Msg::HandleKeyDown(key) => {
                use MoveDirection::*;
                match key.key().as_ref() {
                    "N" | "n" => self.link.send_self(Msg::FetchNextRoom(N)),
                    "E" | "e" => self.link.send_self(Msg::FetchNextRoom(E)),
                    "W" | "w" => self.link.send_self(Msg::FetchNextRoom(W)),
                    "S" | "s" => self.link.send_self(Msg::FetchNextRoom(S)),
                    "Escape" => self.link.send_self(Msg::ClearNotifications),
                    _ => {},
                }
            }
            Msg::FetchNextRoom(direction) => {
                if self.loading() || !self.state.can_move_direction(direction) {
                    return false;
                }
                let status = self.state.status.clone();
                match status {
                    Status::Loading => error!("Logic error: no current room."),
                    Status::InRoom(current_room_id) => {
                        self.fetch(FetchRoomRequest::NextRoom(current_room_id, direction));
                    }
                    Status::Finished(_) => error!("Logic error: no more room."),
                }
            }
            Msg::ReceivedRoom(room, last_move) => {
                self.fetching = false;
                self.fetching_move = None;

                let room_id = room.location_path.clone();
                self.state.insert_room(room, last_move);
                self.link.send_self(Msg::MoveToRoom(room_id));
            }
            Msg::MoveToRoom(room_id) => {
                self.state.status = Status::InRoom(room_id);
                self.state.draw_map();
            }
            Msg::ReceivedMessage(message) => {
                self.fetching = false;
                self.fetching_move = None;

                self.link.send_self(Msg::NewNotification(Notification {
                    message: format!("{}", message.message),
                    level: NotificationLevel::Warning,
                }));
            }
            Msg::ReceivedExit(exit, last_move) => {
                self.fetching = false;
                self.fetching_move = None;

                let room_id = self.state.reached_exit(exit, last_move);
                self.link.send_self(Msg::MoveToRoom(room_id));

                self.link.send_self(Msg::NewNotification(Notification {
                    message: "Congratulations! You exited the maze!".to_string(),
                    level: NotificationLevel::Success,
                }));
            }
            Msg::FetchRoomFailed(response) => {
                self.fetching = false;
                self.fetching_move = None;
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
            Msg::ClearNotifications => {
                self.notifications.clear();
            }
            Msg::Noop => {
                return false;
            }
        }
        true
    }
}

impl Renderable<Model> for Model {
    fn view(&self) -> Html<Self> {
        let exit_hint = self.state.current_exit_hint();
        let exited = self.state.exited();
        html! {
            <section>
                { self.view_notifications() }
                <components::Compass: maze_exit_hint=exit_hint exited=exited/>
                { self.view_room() }
                { self.view_buttons() }
                { self.view_map() }
            </section>
        }
    }
}

// Accessors
impl Model {
    fn loading(&self) -> bool {
        self.fetching || self.state.status == Status::Loading
    }
}

// Views
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
        let view_exit = |(idx, direction): (usize, &MoveDirection)| {
            html! {
                <span>
                    { if idx > 0 { ", " } else { "" } }
                    { direction.long_name() }
                </span>
            }
        };
        match &self.state.status {
            Status::Loading => html! { <h1>{ "Loading first room..." }</h1> },
            Status::InRoom(room_id) => {
                if let Some((room, _coord)) = self.state.rooms.get(room_id) {
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
                                <span>
                                    { for room.exits.iter().enumerate().map(view_exit) }
                                </span>
                                { "." }
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
            }
            Status::Finished(exit) => html! {
                <div>
                    <p id="status">{ status_to_str(exit.status) }</p>
                    <p id="description">{ &exit.description }</p>
                </div>
            },
        }
    }

    fn view_buttons(&self) -> Html<Model> {
        let loading = self.loading();

        use MoveDirection::*;
        let button = |direction: MoveDirection| {
            let can = self.state.can_move_direction(direction) && !loading;
            match can {
                true => html! {
                    <button class="btn btn--primary btn--large" style="margin-left: 5px;"
                        onclick=|_| Msg::FetchNextRoom(direction)>
                        { "Go " }{ direction.long_name() }
                    </button>
                },
                false => html! {
                    <button class="btn btn--inverted btn--large" style="margin-left: 5px;">
                        { "Go " }{ direction.long_name() }
                    </button>
                },
            }
        };
        html! {
            <div id="buttons">
                { for [W, N, S, E].iter().cloned().map(button) }
                // TODO: Restart feature
                //<button class="btn btn--primary" style="margin-left: 5px;"
                //    onclick=|_| Msg::Init>
                //    { "Restart" }
                //</button>
            </div>
        }
    }

    fn view_map(&self) -> Html<Model> {
        const DISPLAY_NONE: &'static str = "display: none";
        const MAP_BORDER: &'static str = "border: 2px solid black";
        let (div_style, map_style) = match &self.state.status {
            Status::Loading => (DISPLAY_NONE, ""),
            Status::InRoom(_) => ("", MAP_BORDER),
            Status::Finished(_) => ("", MAP_BORDER),
        };
        html! {
            <div style=div_style>
                <h3>{ "Map" }</h3>
                <canvas id="pathbot-map-canvas"
                    style=map_style
                    width="500" height="300"></canvas>
            </div>
        }
    }
}

// Fetch
impl Model {
    fn fetch(&mut self, request: FetchRoomRequest) {
        if self.fetching {
            warn!("Not sending, ongoing request.");
            return;
        }
        self.fetching = true;
        self.fetching_move = None;

        // Save the fetched direction
        self.fetching_move = match &request {
            FetchRoomRequest::StartRoom => None,
            FetchRoomRequest::NextRoom(_, dir) => Some(*dir),
        };

        // Check if the room is cached
        match &request {
            FetchRoomRequest::NextRoom(_, move_direction) => {
                // Compute the next room's coordinates
                let cur = self
                    .state
                    .current_coordinates()
                    .expect("Logic error: must have a room");
                let next_coords = cur + move_direction.delta();

                if let Some(room_id) = self.state.coord_to_id.get(&next_coords) {
                    self.fetching = false;
                    self.link.send_self(Msg::MoveToRoom(room_id.clone()));
                    return;
                };
            }
            _ => {}
        }

        // Build the request
        let (request, last_move): (Request<Text>, _) = match request {
            FetchRoomRequest::StartRoom => (
                Request::post("https://api.noopschallenge.com/pathbot/start")
                    .header("Content-Type", "application/json")
                    .body(Ok("".to_string()))
                    .unwrap(), // cannot really fail (except OOM)
                None,
            ),
            FetchRoomRequest::NextRoom(location_path, move_direction) => {
                let url = format!("https://api.noopschallenge.com{}", location_path);
                let body = json!({ "direction": move_direction });
                (
                    Request::post(url)
                        .header("Content-Type", "application/json")
                        .body(Json(&body).into())
                        .unwrap(), // cannot really fail (except OOM)
                    Some(move_direction),
                )
            }
        };

        // Send the request
        let callback = self
            .link
            .send_back(move |response: Response<Json<Result<_, Error>>>| {
                let (_meta, Json(data)) = response.into_parts();
                match data {
                    Ok(PathbotApiMessage::Room(room)) => Msg::ReceivedRoom(room, last_move),
                    Ok(PathbotApiMessage::Message(message)) => Msg::ReceivedMessage(message),
                    Ok(PathbotApiMessage::Exit(exit)) => Msg::ReceivedExit(exit, last_move),
                    Err(e) => Msg::FetchRoomFailed(e),
                }
            });
        let task = self.fetch_service.fetch(request, callback);
        self.fetch_task = Some(task);
    }
}

impl State {
    fn restart(&mut self) {
        self.status = Status::Loading;
        self.rooms.clear();
    }

    fn exited(&self) -> bool {
        match &self.status {
            Status::Finished(_) => true,
            _ => false,
        }
    }

    fn current_exit_hint(&self) -> Option<MazeExitHint> {
        match &self.status {
            Status::InRoom(id) => self
                .rooms
                .get(id)
                .map(|t| &t.0)
                .map(|r| r.maze_exit_hint.clone()),
            _ => None,
        }
    }

    fn current_exits(&self) -> Option<&Vec<MoveDirection>> {
        match &self.status {
            Status::InRoom(id) => self.rooms.get(id).map(|t| &t.0.exits),
            _ => None,
        }
    }

    fn current_coordinates(&self) -> Option<Coordinate> {
        match &self.status {
            Status::InRoom(id) => self.rooms.get(id).map(|t| t.1.clone()),
            _ => None,
        }
    }

    fn current_room_id(&self) -> Option<&RoomId> {
        match &self.status {
            Status::InRoom(id) => Some(id),
            _ => None,
        }
    }

    fn can_move_direction(&self, direction: MoveDirection) -> bool {
        if let Some(current_exits) = self.current_exits() {
            current_exits.contains(&direction)
        } else {
            false
        }
    }

    fn insert_room(&mut self, room: Room, last_move: Option<MoveDirection>) {
        // TODO: There are too many clone here
        let location_path = room.location_path.clone();

        // Save the room, with its position
        let position = match last_move {
            Some(prev_move) => {
                let prev_id = match &self.status {
                    Status::InRoom(id) => id,
                    _ => panic!(format!(
                        "Logic error: cannot insert room when status == {:?}.",
                        self.status
                    )),
                };
                let prev_position = self
                    .rooms
                    .get(prev_id)
                    .cloned()
                    .expect("Logic error: room must exist.")
                    .1;
                prev_position + prev_move.delta()
            }
            None => Coordinate { x: 0, y: 0 },
        };
        self.rooms
            .insert(room.location_path.clone(), (room, position));

        // Add to the caches
        self.room_coords.insert(location_path.clone(), position);
        self.coord_to_id.insert(position, location_path);
    }

    fn reached_exit(&mut self, exit: Exit, last_move: Option<MoveDirection>) -> String {
        let exit_move = match last_move.expect("Logic error: we must have moved here.") {
            MoveDirection::N => MoveDirection::S,
            MoveDirection::S => MoveDirection::N,
            MoveDirection::E => MoveDirection::W,
            MoveDirection::W => MoveDirection::E,
        };

        // Fake a room (for the map)
        // TODO: Shouldn't have to fake a room
        let exit_room_id = "exit_room_yay".to_string();
        let room = Room {
            status: RoomStatus::Finished,
            message: "Thank you for playing :)".to_string(),
            exits: vec![exit_move],
            description: exit.description.clone(),
            maze_exit_hint: MazeExitHint {
                // TODO: Should be None
                direction: CompassDirection::N,
                distance: 0,
            },
            // TODO: Should be None
            location_path: exit_room_id.clone(),
        };
        self.insert_room(room, last_move);

        self.status = Status::Finished(exit);

        exit_room_id
    }

    fn draw_map(&self) {
        let canvas: CanvasElement = document()
            .query_selector("#pathbot-map-canvas")
            .unwrap()
            .expect("Didn't find the map canvas.")
            .try_into() // Element -> CanvasElement
            .unwrap(); // cannot be other than a canvas
        let context: CanvasRenderingContext2d = canvas.get_context().unwrap();

        context.clear_rect(0., 0., canvas.width() as f64, canvas.height() as f64);

        const ROOM_W: f64 = 20.;
        const ROOM_H: f64 = 20.;
        const EXIT_L: f64 = 5.;
        const EXIT_LW: f64 = 2.;
        const SHIFT_X: f64 = ROOM_W / 2.;
        const SHIFT_Y: f64 = ROOM_H / 2.;

        context.set_line_width(EXIT_LW);

        let current_room_id = self
            .current_room_id()
            .expect("Logic error: must have a current room.");
        let current_coordinates = self
            .current_coordinates()
            .expect("Logic error: must have a current room.");

        let offset_x =
            canvas.width() as f64 / 2. - current_coordinates.x as f64 * (ROOM_W + EXIT_L);
        let offset_y =
            canvas.height() as f64 / 2. - current_coordinates.y as f64 * (ROOM_H + EXIT_L);

        // Draw the exits
        context.begin_path();
        context.set_fill_style_color("black");
        for (_, (room, Coordinate { x, y })) in &self.rooms {
            let origin_x = offset_x + (*x as f64) * (ROOM_W + EXIT_L);
            let origin_y = offset_y + (*y as f64) * (ROOM_H + EXIT_L);

            for exit in &room.exits {
                use MoveDirection::*;
                let (from, to) = match exit {
                    N => ((0., -SHIFT_Y), (0., -SHIFT_Y - EXIT_L)),
                    W => ((-SHIFT_X, 0.), (-SHIFT_X - EXIT_L, 0.)),
                    E => ((SHIFT_X, 0.), (SHIFT_X + EXIT_L, 0.)),
                    S => ((0., SHIFT_Y), (0., SHIFT_Y + EXIT_L)),
                };
                context.move_to(origin_x + from.0, origin_y + from.1);
                context.line_to(origin_x + to.0, origin_y + to.1);
            }
        }
        context.stroke();

        // Draw the rooms
        for (id, (room, Coordinate { x, y })) in &self.rooms {
            let room_color = if *x == 0 && *y == 0 {
                "blue" // initial
            } else if room.status == RoomStatus::Finished {
                "green" // exit
            } else if id == current_room_id {
                "red" // current
            } else {
                "pink" // all other
            };

            context.set_fill_style_color(room_color);
            let origin_x = offset_x + (*x as f64) * (ROOM_W + EXIT_L);
            let origin_y = offset_y + (*y as f64) * (ROOM_H + EXIT_L);
            context.fill_rect(
                origin_x - ROOM_W / 2.,
                origin_y - ROOM_H / 2.,
                ROOM_W,
                ROOM_H,
            );
        }
    }
}

impl MoveDirection {
    fn delta(&self) -> Coordinate {
        match &self {
            MoveDirection::N => Coordinate { x: 0, y: -1 },
            MoveDirection::S => Coordinate { x: 0, y: 1 },
            MoveDirection::W => Coordinate { x: -1, y: 0 },
            MoveDirection::E => Coordinate { x: 1, y: 0 },
        }
    }
}
