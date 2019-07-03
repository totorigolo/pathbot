use pathbot::{Model, Msg};
use log::trace;
use yew::App;

fn main() {
    web_logger::init();
    trace!("Initializing yew...");
    yew::initialize();
    App::<Model>::new()
        .mount_to_body()
        .send_message(Msg::Init);
    yew::run_loop();
}
