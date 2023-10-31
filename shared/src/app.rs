use crate::capabilities::passkey::Passkey;
use crux_core::render::Render;
use crux_macros::Effect;
use log::info;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum Event {
    None,

    //driving...
    Register(String),
    Login(String),

    // driven...
    #[serde(skip)]
    Registered,
    #[serde(skip)]
    LoggedIn,
    #[serde(skip)]
    Error(String),
}

#[derive(Default, Clone, Debug, PartialEq, Eq)]
enum State {
    #[default]
    Steady,
    Registering,
    LoggingIn,
}

#[derive(Serialize, Deserialize, Default, Clone, Debug, PartialEq, Eq)]
pub enum Status {
    #[default]
    None,
    Info(String),
    Error(String),
}

#[derive(Default, Debug)]
pub struct Model {
    user_name: String,
    state: State,
    status: Status,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub status: Status,
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
pub struct Capabilities {
    render: Render<Event>,
    passkey: Passkey<Event>,
}

#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        match (model.state.clone(), event.clone()) {
            (_, Event::None) => {}
            (State::Steady, Event::Register(user_name)) => {
                if user_name.is_empty() {
                    model.status = Status::Error("user name cannot be empty".to_string());
                } else {
                    info!("registering user: {}", user_name);
                    model.user_name = user_name.clone();
                    model.state = State::Registering;
                    model.status = Status::Info("registering...".to_string());
                    caps.passkey.register(user_name, Event::Registered);
                }
            }
            (State::Registering, Event::Registered) => {
                model.state = State::Steady;
                model.status = Status::Info("registered".to_string());
            }
            (State::Steady, Event::Login(user_name)) => {
                if user_name.is_empty() {
                    model.status = Status::Error("user name cannot be empty".to_string());
                } else {
                    info!("logging in user: {}", user_name);
                    model.user_name = user_name.clone();
                    model.state = State::LoggingIn;
                    model.status = Status::Info("logging in...".to_string());
                    caps.passkey.login(user_name, Event::LoggedIn);
                }
            }
            (State::LoggingIn, Event::LoggedIn) => {
                model.state = State::Steady;
                model.status = Status::Info("logged in".to_string());
            }
            (_, Event::Error(e)) => {
                model.state = State::Steady;
                model.status = Status::Error(e);
            }
            (s, m) => {
                info!("Invalid State Transition -> {s:?}, {m:?}");
            }
        };

        info!("update: {:?} {:?}", event, model);
        caps.render.render();
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            status: model.status.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::passkey::{PasskeyOperation, PasskeyOutput};
    use assert_let_bind::assert_let;
    use crux_core::{assert_effect, testing::AppTester};

    #[test]
    fn registration() {
        let app = AppTester::<App, _>::default();

        let mut model = Model::default();

        let event = Event::Register("stu".to_string());

        let mut update = app.update(event, &mut model);
        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Info: registering...
        "###);

        assert_let!(Effect::Passkey(request), &mut update.effects[0]);

        let actual = &request.operation;
        let expected = &PasskeyOperation::Register("stu".to_string());
        assert_eq!(actual, expected);

        // simulate a successful response from the server
        let response = PasskeyOutput::Registered;
        let update = app.resolve(request, response).expect("an update");

        let actual = update.events[0].clone();
        let expected = Event::Registered;
        assert_eq!(actual, expected);

        let update = app.update(actual, &mut model);

        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Info: registered
        "###);
    }

    #[test]
    fn registering_with_empty_username() {
        let app = AppTester::<App, _>::default();

        let mut model = Model::default();

        let event = Event::Register("".to_string());

        let update = app.update(event, &mut model);
        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Error: user name cannot be empty
        "###);
    }

    #[test]
    fn login() {
        let app = AppTester::<App, _>::default();

        let mut model = Model::default();

        let event = Event::Login("stu".to_string());

        let mut update = app.update(event, &mut model);
        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Info: logging in...
        "###);

        assert_let!(Effect::Passkey(request), &mut update.effects[0]);

        let actual = &request.operation;
        let expected = &PasskeyOperation::Login("stu".to_string());
        assert_eq!(actual, expected);

        // simulate a successful response from the server
        let response = PasskeyOutput::LoggedIn;
        let update = app.resolve(request, response).expect("an update");

        let actual = update.events[0].clone();
        let expected = Event::LoggedIn;
        assert_eq!(actual, expected);

        let update = app.update(actual, &mut model);

        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Info: logged in
        "###);
    }

    #[test]
    fn logging_in_with_empty_username() {
        let app = AppTester::<App, _>::default();

        let mut model = Model::default();

        let event = Event::Login("".to_string());

        let update = app.update(event, &mut model);
        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Error: user name cannot be empty
        "###);
    }
}
