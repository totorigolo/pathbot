use log::*;
use yew::prelude::*;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

use crate::Notification as NotificationData;
use crate::NotificationLevel;

pub struct Notification {
    props: Props,
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
        Notification { props }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::Closed => match self.props.on_close {
                Some(ref mut callback) => callback.emit(()),
                None => error!("No callback on notification."),
            },
        }
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl Renderable<Notification> for Notification {
    fn view(&self) -> Html<Self> {
        let notif_class = match self.props.notification.level {
            NotificationLevel::Info => "notice--info",
            NotificationLevel::Success => "notice--success",
            NotificationLevel::Warning => "notice--warning",
            NotificationLevel::Danger => "notice--danger",
        };
        html! {
            <div class=notif_class>
                { &self.props.notification.message }
                <button style="float: right" class="btn btn--primary"
                    onclick=|_| Msg::Closed>{ "x" }</button>
            </div>
        }
    }
}
