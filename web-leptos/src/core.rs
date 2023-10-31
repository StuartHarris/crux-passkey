use std::rc::Rc;

use leptos::{spawn_local, SignalUpdate, WriteSignal};
use shared::{App, Capabilities, Effect, Event, ViewModel};

use crate::passkey;

pub type Core = Rc<shared::Core<Effect, App>>;

pub fn new() -> Core {
    Rc::new(shared::Core::new::<Capabilities>())
}

pub fn update(core: &Core, event: Event, render: WriteSignal<ViewModel>) {
    for effect in core.process_event(event) {
        process_effect(core, effect, render);
    }
}

pub fn process_effect(core: &Core, effect: Effect, render: WriteSignal<ViewModel>) {
    match effect {
        Effect::Render(_) => {
            render.update(|view| *view = core.view());
        }
        Effect::Passkey(mut request) => {
            spawn_local({
                let core = core.clone();

                async move {
                    let response = passkey::request(&request.operation).await.unwrap();

                    for effect in core.resolve(&mut request, response) {
                        process_effect(&core, effect, render);
                    }
                }
            });
        }
    };
}
