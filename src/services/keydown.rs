// From: https://github.com/s3k/yew-keydown-example/blob/master/src/keydown_service.rs
//
use log::*;
use stdweb::web::event::KeyDownEvent;
use stdweb::Value;
use yew::callback::Callback;
use yew::services::Task;

#[must_use]
pub struct KeydownTask(Option<Value>);

#[derive(Default)]
pub struct KeydownService {}

impl KeydownService {
    pub fn new() -> Self {
        Self {}
    }

    pub fn spawn(&mut self, callback: Callback<KeyDownEvent>) -> KeydownTask {
        let callback = move |e| {
            callback.emit(e);
        };

        let handle = js! {
            var callback = @{callback};

            var action = function(e) {
                callback(e);
            };

            window.addEventListener("keydown", action);

            return {
                callback: callback,
            };
        };

        KeydownTask(Some(handle))
    }
}

impl Task for KeydownTask {
    fn is_active(&self) -> bool {
        self.0.is_some()
    }

    fn cancel(&mut self) {
        let handle = self
            .0
            .take()
            .expect("tried to cancel window keydown listener");

        // This not working. Suggest your solution.
        warn!("Dropping KeydownTask doesn't really work.");
        js! { @(no_return)
            var handle = @{handle};
            window.removeEventListener("keydown", handle.callback);
            handle.callback.drop();
        }
    }
}

impl Drop for KeydownTask {
    fn drop(&mut self) {
        if self.is_active() {
            self.cancel();
        }
    }
}
