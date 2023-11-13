mod core;
mod http;
mod passkey;

use leptos::{
    component, create_effect, create_node_ref, create_signal, ev::SubmitEvent, event_target_value,
    html::Input, view, window, IntoView, NodeRef, SignalGet, SignalUpdate,
};
use shared::{Event, Status};

#[component]
fn RootComponent() -> impl IntoView {
    let core = core::new();
    let (view, render) = create_signal(core.view());
    let (event, set_event) = create_signal(Event::ServerUrl(
        window().location().origin().expect("origin to exist"),
    ));

    create_effect(move |_| {
        core::update(&core, event.get(), render);
    });

    let input_element: NodeRef<Input> = create_node_ref();
    let on_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        let Some(submitter) = ev.submitter() else {
            return;
        };

        let user_name = input_element.get().expect("<input> to exist").value();
        match submitter.id().as_ref() {
            "register" => {
                set_event.update(|value| *value = Event::Register(user_name));
            }
            "login" => {
                set_event.update(|value| *value = Event::Login(user_name));
            }
            _ => {}
        }
    };

    let notification = move || match view.get().status {
        Status::None => {
            view! {<div />}
        }
        Status::Info(msg) => {
            view! {<div class="notification is-info is-light">{msg}</div>}
        }
        Status::Error(msg) => {
            view! {<div class="notification is-warning is-light">{msg}</div>}
        }
    };
    view! {
        <section class="box container has-text-centered m-5">
            <div class="container" style="max-width:400px">
                <form on:submit=on_submit>
                    <input class="input is-primary" type="text" placeholder="user name"
                        node_ref=input_element
                        on:input=move |ev| {
                            set_event.update(|value| *value = Event::Validate(event_target_value(&ev)));
                        }
                    />
                    <div class="m-2">{notification}</div>
                    <div class="buttons section is-centered">
                        <button id="register" type="submit" class="button is-primary is-warning">
                        {"Register"}
                        </button>
                        <button id="login" type="submit" class="button is-primary is-success">
                        {"Login"}
                        </button>
                    </div>
                </form>
            </div>
        </section>
    }
}

fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    leptos::mount_to_body(|| {
        view! { <RootComponent /> }
    });
}
