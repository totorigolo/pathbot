use pathbot::{Model, Msg};
use yew::App;
use stdweb::web::{document, IParentNode};

fn main() {
    web_logger::init();
    yew::initialize();

    let mount_point = document()
        .query_selector("#pathbot-root")
        .expect("can't find #pathbot-root node for mounting app")
        .expect("can't unwrap #pathbot-root node");

    App::<Model>::new()
        .mount(mount_point)
        .send_message(Msg::Init);

    yew::run_loop();
}
