use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};
use log::*;

use crate::{CompassDirection, MazeExitHint};

pub struct Compass {
    maze_exit_hint: Option<MazeExitHint>,
}

pub enum Msg {
}

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
        use CompassDirection::*;
        if let Some(exit_hint) = self.maze_exit_hint {
            let direction = match exit_hint.direction {
                N => "North",
                S => "South",
                E => "East",
                W => "West",
                NW => "North-West",
                NE => "North-East",
                SW => "South-West",
                SE => "South-East",
            };
            let angle = 180. * match exit_hint.direction {
                E => 0.,
                NE => 1./4.,
                N => 1./2.,
                NW => 3./4.,
                W => 1.,
                SW => 5./4.,
                S => 3./2.,
                SE => 7./4.,
            };
            let rotate_style = format!("transform: rotate({}deg); width:100px; height:100px;", angle);
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


