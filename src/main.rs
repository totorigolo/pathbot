#![feature(try_blocks)]

#[macro_use]
extern crate stdweb;
extern crate serde_json;

use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

use stdweb::traits::*;
use stdweb::unstable::TryInto;
use stdweb::web::{document, window, Element, HtmlElement, XmlHttpRequest};

use stdweb::web::event::{BlurEvent, ChangeEvent, ClickEvent, DoubleClickEvent, HashChangeEvent, KeyPressEvent, ProgressLoadEvent};

use stdweb::web::html_element::InputElement;

// Shamelessly stolen from webplatform's TodoMVC example.
macro_rules! enclose {
    ( ($( $x:ident ),*) $y:expr ) => {
        {
            $(let $x = $x.clone();)*
            $y
        }
    };
}

macro_rules! query {
    ($selector:expr) => {
        document()
            .query_selector($selector)
            .expect(concat!("Invalid syntax: ", stringify!($selector),"."))
            .expect(concat!("No element found: ", stringify!($selector),"."))
    };
    ($selector:expr, $into_ty:ty) => {
        (query!($selector)
            .try_into() as Result<$into_ty, _>)
            .expect(concat!(stringify!($selector), " isn't a ", stringify!($into_ty)))
    };
}

#[derive(Serialize, Deserialize)]
struct State {
    title: String,
    status: String,
    message: String,
}

impl State {
    fn new() -> Self {
        State {
            title: "The title".to_string(),
            status: "Wonderful status".to_string(),
            message: "Very interesting message".to_string(),
        }
    }
}

type StateRef = Rc<RefCell<State>>;

fn update_dom(state: &StateRef) {
    let state_borrow = state.borrow();

    let title: Element = query!("#title");
    title.set_text_content(&state_borrow.title);

    let status: Element = query!("#status");
    status.set_text_content(&state_borrow.status);

    let message: Element = query!("#message");
    message.set_text_content(&state_borrow.message);
}

/// Save the state into local storage.
fn save_state(state: &StateRef) {
    //-> Result<(), Box<dyn std::error::Error>> {
    let result: Result<(), Box<dyn std::error::Error>> = try {
        let state_json = serde_json::to_string(&*state.borrow())?;
        window()
            .local_storage()
            .insert("state", state_json.as_str())?;
    };
    if let Err(error) = result {
        console!(
            error,
            format!("Failed to save the state in local storage: {}", error)
        );
    }
}

/// Load the state from local storage.
fn load_state() -> State {
    window()
        .local_storage()
        .get("state")
        .and_then(|state_json| serde_json::from_str(state_json.as_str()).ok())
        .unwrap_or_else(State::new)
}

fn main() {
    stdweb::initialize();
    console!(log, "Loading...");

    let state = load_state();
    let state: StateRef = Rc::new(RefCell::new(state));

    let input: InputElement = query!("#input", InputElement);
    input.add_event_listener(enclose!( (state, input) move |event: KeyPressEvent| {
        if event.key() == "Enter" {
            event.prevent_default();

            state.borrow_mut().title = input.raw_value();
            state.borrow_mut().status = input.raw_value();
            input.set_raw_value("");

            update_dom(&state);
            save_state(&state);
        }
    }));

    let xhr = XmlHttpRequest::new();
    xhr.open("POST", "https://api.noopschallenge.com/pathbot/start");
    xhr.add_event_listener( enclose!( (xhr, state) move |e: ProgressLoadEvent| {
        console!(log, "XHR progress: ", e);
        let response = xhr.response_text().unwrap().unwrap();

        state.borrow_mut().message = response;
        update_dom(&state);
        save_state(&state);
    }));
    xhr.send();

    update_dom(&state);
    stdweb::event_loop();
}
