use pathbot::{Model, Msg};
use yew::App;

fn main() {
    web_logger::init();

    yew::initialize();

    App::<Model>::new()
        .mount_to_body()
        .send_message(Msg::Init);

    yew::run_loop();
}
