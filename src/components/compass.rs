use log::*;
use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

use crate::MazeExitHint;

pub struct Compass {
    maze_exit_hint: Option<MazeExitHint>,
}

pub enum Msg {}

#[derive(PartialEq, Clone)]
pub struct Props {
    pub maze_exit_hint: Option<MazeExitHint>,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            maze_exit_hint: None,
        }
    }
}

impl Component for Compass {
    type Message = Msg;
    type Properties = Props;

    fn create(_: Self::Properties, _: ComponentLink<Self>) -> Self {
        trace!("Compass - created");
        Compass {
            maze_exit_hint: None,
        }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        trace!("Compass - update");
        // match msg {}
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        trace!("Compass - change");
        self.maze_exit_hint = props.maze_exit_hint;
        true
    }
}

impl Renderable<Compass> for Compass {
    fn view(&self) -> Html<Self> {
        if let Some(exit_hint) = self.maze_exit_hint {
            let direction = exit_hint.direction.long_name();
            let angle = exit_hint.direction.angle_deg(); // clockwise
            let rotate_style = format!(
                "transform: rotate({}deg); \
                 width:100px; \
                 height:100px;",
                angle
            );
            html! {
                <div div="compass", style=rotate_style,>
                    <p>{ format!("Direction: {}", direction) }</p>
                    <p>{ format!("Distance: {}", exit_hint.distance) }</p>
                    <p>{ format!("Angle: {}", angle) }</p>
                </div>
            }
        } else {
            html! {
                <div div="compass",>
                    <p>{ "No compass" }</p>
                </div>
            }
        }
    }
}
