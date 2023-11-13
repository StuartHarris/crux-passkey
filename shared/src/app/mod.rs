mod auth;

use crux_core::{render::Render, Capability};
use crux_http::Http;
use crux_macros::Effect;
use serde::{Deserialize, Serialize};

use crate::passkey::Passkey;
pub use auth::Status;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    // driving...
    ServerUrl(String),
    Validate(String),
    Register(String),
    Login(String),

    // driven...
    #[serde(skip)]
    Auth(auth::Event),
}

#[derive(Default, Debug)]
pub struct Model {
    auth: auth::Model,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub status: auth::Status,
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
pub struct Capabilities {
    pub http: Http<Event>,
    pub passkey: Passkey<Event>,
    pub render: Render<Event>,
}

impl From<&Capabilities> for auth::Capabilities {
    fn from(caps: &Capabilities) -> Self {
        Self {
            http: caps.http.map_event(Event::Auth),
            passkey: caps.passkey.map_event(Event::Auth),
            render: caps.render.map_event(Event::Auth),
        }
    }
}

#[derive(Default)]
pub struct App {
    auth: auth::App,
}

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match event {
            Event::ServerUrl(e) => {
                self.auth
                    .update(auth::Event::ServerUrl(e), &mut model.auth, &caps.into())
            }
            Event::Validate(e) => {
                self.auth
                    .update(auth::Event::Validate(e), &mut model.auth, &caps.into())
            }
            Event::Register(e) => {
                self.auth
                    .update(auth::Event::Register(e), &mut model.auth, &caps.into())
            }
            Event::Login(e) => {
                self.auth
                    .update(auth::Event::Login(e), &mut model.auth, &caps.into())
            }
            Event::Auth(event) => {
                self.auth.update(event, &mut model.auth, &caps.into());
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            status: model.auth.status.clone(),
        }
    }
}
