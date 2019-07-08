use log::*;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use yew::prelude::*;

use crate::Notification as NotificationData;
use crate::NotificationLevel;

pub struct Notification {
    data: NotificationData,
    on_close: Option<Callback<()>>,
}

pub enum Msg {
    Closed,
}

#[derive(PartialEq, Clone)]
pub struct Props {
    pub notification: NotificationData,
    pub on_close: Option<Callback<()>>,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            notification: NotificationData {
                message: "".to_string(),
                level: NotificationLevel::Info,
            },
            on_close: None,
        }
    }
}

impl Component for Notification {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Notification {
            data: props.notification,
            on_close: props.on_close,
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Closed => {
                match self.on_close {
                    Some(ref mut callback) => callback.emit(()),
                    None => error!("No callback on notification."),
                }
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.data = props.notification;
        self.on_close = props.on_close;
        true
    }
}

impl Renderable<Notification> for Notification {
    fn view(&self) -> Html<Self> {
        let class = match self.data.level {
            NotificationLevel::Info => "notification is-info",
            NotificationLevel::Success => "notification is-success",
            NotificationLevel::Warning => "notification is-warning",
            NotificationLevel::Danger => "notification is-danger",
        };
        html! {
            <div class=class,>
                <button class="delete", onclick=|_| Msg::Closed,></button>
                { &self.data.message }
            </div>
        }
    }
}
