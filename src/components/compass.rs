use yew::{html, Component, ComponentLink, Html, Renderable, ShouldRender};

use crate::MazeExitHint;

pub struct Compass {
    props: Props,
}

pub enum Msg {}

#[derive(PartialEq, Clone)]
pub struct Props {
    pub maze_exit_hint: Option<MazeExitHint>,
    pub exited: bool,
}

impl Default for Props {
    fn default() -> Self {
        Props {
            maze_exit_hint: None,
            exited: false,
        }
    }
}

impl Component for Compass {
    type Message = Msg;
    type Properties = Props;

    fn create(props: Self::Properties, _: ComponentLink<Self>) -> Self {
        Compass { props }
    }

    fn update(&mut self, _msg: Self::Message) -> ShouldRender {
        // match msg {}
        true
    }

    fn change(&mut self, props: Self::Properties) -> ShouldRender {
        self.props = props;
        true
    }
}

impl Renderable<Compass> for Compass {
    fn view(&self) -> Html<Self> {
        let compass_style = "float: right; margin: 10px;";
        let (direction, angle, distance) = match (self.props.maze_exit_hint, self.props.exited) {
            (Some(exit_hint), _) => (
                exit_hint.direction.long_name(),
                exit_hint.direction.angle_deg(), // clockwise
                Some(exit_hint.distance),
            ),
            (None, false) => ("?", 0., None),
            (None, true) => ("-", 0., Some(0)),
        };
        let rotate_style = format!(
            "transform: rotate({}deg); \
             width:100px; \
             height:100px;",
            angle
        );
        let distance_str = distance
            .map(|d| format!("{}", d))
            .unwrap_or("?".to_string());
        html! {
            <div class="compass" style=compass_style>
                <img src="compass.png" style=rotate_style />
                <p>
                    { "Direction: "}{ direction }
                    <br />
                    { "Distance: "}{ distance_str }
                </p>
            </div>
        }
    }
}
