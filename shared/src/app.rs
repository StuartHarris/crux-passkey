use crate::capabilities::passkey::Passkey;
use crux_core::render::Render;
use crux_http::Http;
use crux_macros::Effect;
use log::info;
use serde::{Deserialize, Serialize};
use webauthn_rs_proto::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

const SERVER_URL: &str = "https://crux-passkey-server-9uqexpm2.fermyon.app"; // todo: config

// const SERVER_URL: &str = "https://localhost"; // todo: config

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Event {
    None,

    // driving...
    Validate(String),
    Register(String),
    Login(String),

    // driven...
    #[serde(skip)]
    GetCreationChallenge(String), // register
    #[serde(skip)]
    GetRequestChallenge(String), // login

    #[serde(skip)]
    CreationChallenge(crux_http::Result<crux_http::Response<CreationChallengeResponse>>), // register
    #[serde(skip)]
    RequestChallenge(crux_http::Result<crux_http::Response<RequestChallengeResponse>>), // login

    #[serde(skip)]
    RegisterCredential(RegisterPublicKeyCredential), // register
    #[serde(skip)]
    Credential(PublicKeyCredential), // login

    #[serde(skip)]
    CredentialRegistered(crux_http::Result<crux_http::Response<Vec<u8>>>), // register
    #[serde(skip)]
    CredentialVerified(crux_http::Result<crux_http::Response<Vec<u8>>>), // login

    #[serde(skip)]
    Error(String),
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
    status: Status,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ViewModel {
    pub status: Status,
}

#[cfg_attr(feature = "typegen", derive(crux_macros::Export))]
#[derive(Effect)]
pub struct Capabilities {
    http: Http<Event>,
    passkey: Passkey<Event>,
    render: Render<Event>,
}

#[derive(Default)]
pub struct App;

impl crux_core::App for App {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Capabilities = Capabilities;

    fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
        info!("update: {:?}", event);
        match event {
            Event::None => {}
            Event::Validate(user_name) => {
                model.status = if user_name.is_empty() {
                    Status::Error("user name cannot be empty".to_string())
                } else {
                    Status::None
                };
                caps.render.render();
            }
            Event::Register(user_name) => {
                self.update(Event::Validate(user_name.clone()), model, caps);
                if model.status == Status::None {
                    info!("registering user: {}", user_name);
                    model.user_name = user_name.clone();
                    model.status = Status::Info(format!(r#"registering "{user_name}"..."#));
                    caps.render.render();
                    self.update(Event::GetCreationChallenge(user_name), model, caps);
                }
            }
            Event::GetCreationChallenge(user_name) => {
                info!("getting creation challenge for user: {}", user_name);
                caps.http
                    .get(format!("{SERVER_URL}/auth/register_start/{}", user_name))
                    .expect_json()
                    .send(Event::CreationChallenge);
            }
            Event::CreationChallenge(Ok(mut response)) => {
                let ccr = response.take_body().expect("http response has a body");
                let bytes = serde_json::to_vec(&ccr).expect("json serializable");
                info!("ask authenticator to create credential");
                caps.passkey
                    .create_credential(bytes, Event::RegisterCredential);
            }
            Event::CreationChallenge(Err(e)) => {
                self.update(
                    Event::Error(format!("failed to get creation challenge: {:?}", e)),
                    model,
                    caps,
                );
            }
            Event::RegisterCredential(cred) => {
                info!("registering credential");
                caps.http
                    .post(format!("{SERVER_URL}/auth/register_finish"))
                    .body_json(&cred)
                    .expect("json serializable")
                    .send(Event::CredentialRegistered);
            }
            Event::CredentialRegistered(Ok(_)) => {
                model.status = Status::Info(format!(
                    r#"registered "{user_name}""#,
                    user_name = model.user_name
                ));
                caps.render.render();
            }
            Event::CredentialRegistered(Err(e)) => {
                self.update(
                    Event::Error(format!("failed to register: {:?}", e)),
                    model,
                    caps,
                );
            }
            Event::Login(user_name) => {
                self.update(Event::Validate(user_name.clone()), model, caps);
                if model.status == Status::None {
                    info!("logging in user: {}", user_name);
                    model.user_name = user_name.clone();
                    model.status = Status::Info(format!(r#"logging in "{user_name}"..."#));
                    caps.render.render();
                    self.update(Event::GetRequestChallenge(user_name), model, caps);
                }
            }
            Event::GetRequestChallenge(user_name) => {
                info!("getting request challenge for user: {}", user_name);
                caps.http
                    .get(format!("{SERVER_URL}/auth/login_start/{}", user_name))
                    .expect_json()
                    .send(Event::RequestChallenge);
            }
            Event::RequestChallenge(Ok(mut response)) => {
                let rcr = response.take_body().expect("http response has a body");
                let bytes = serde_json::to_vec(&rcr).expect("json serializable");
                info!("ask authenticator to request credential");
                caps.passkey.request_credential(bytes, Event::Credential);
            }
            Event::RequestChallenge(Err(e)) => {
                self.update(
                    Event::Error(format!("failed to get request challenge: {:?}", e)),
                    model,
                    caps,
                );
            }
            Event::Credential(cred) => {
                info!("verifying credential");
                caps.http
                    .post(format!("{SERVER_URL}/auth/login_finish"))
                    .body_json(&cred)
                    .expect("json serializable")
                    .send(Event::CredentialVerified);
            }
            Event::CredentialVerified(Ok(_)) => {
                model.status = Status::Info(format!(
                    r#"logged in "{user_name}""#,
                    user_name = model.user_name
                ));
                caps.render.render();
            }
            Event::CredentialVerified(Err(e)) => {
                self.update(
                    Event::Error(format!("failed to login: {:?}", e)),
                    model,
                    caps,
                );
            }
            Event::Error(e) => {
                model.status = Status::Error(e);
                caps.render.render();
            }
        };
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            status: model.status.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::passkey::PasskeyOperation;

    use super::*;
    use assert_let_bind::assert_let;
    use assert_matches::assert_matches;
    use crux_core::{assert_effect, testing::AppTester};
    use crux_http::protocol::{HttpRequest, HttpResponse};

    #[test]
    fn validation_success() {
        let app = AppTester::<App, _>::default();

        let mut model = Model::default();

        let event = Event::Validate("stu".to_string());

        let update = app.update(event, &mut model);
        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status: None
        "###);
    }

    #[test]
    fn validation_failure() {
        let app = AppTester::<App, _>::default();

        let mut model = Model::default();

        let event = Event::Validate("".to_string());

        let update = app.update(event, &mut model);
        assert_effect!(update, Effect::Render(_));

        insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
        ---
        status:
          Error: user name cannot be empty
        "###);
    }

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
          Info: "registering \"stu\"..."
        "###);

        // check that the app emitted an HTTP request,
        // capturing the request in the process
        assert_let!(Effect::Http(request), &mut update.effects[2]); // 2 renders before this

        // check that the request is a GET to the correct URL
        let actual = &request.operation;
        let expected = &HttpRequest::get(format!("{SERVER_URL}/auth/register_start/stu")).build();
        assert_eq!(actual, expected);

        // resolve the request with a simulated response from the web API
        let response = HttpResponse::ok()
            .body(
                r#"{
                "publicKey": {
                    "rp": {
                        "name": "Crux Passkey",
                        "id": "crux-passkey-server-9uqexpm2.fermyon.app"
                    },
                    "user": {
                        "id": "brs4tqNATYib4pRlF74jSg",
                        "name": "stu",
                        "displayName": "stu"
                    },
                    "challenge": "LnWGR_0kcTrx_qqFPQEZzfsogvic6bSLfXnihBzUYAg",
                    "pubKeyCredParams": [
                        {
                            "type": "public-key",
                            "alg": -7
                        },
                        {
                            "type": "public-key",
                            "alg": -257
                        }
                    ],
                    "timeout": 60000,
                    "attestation": "none",
                    "excludeCredentials": [],
                    "authenticatorSelection": {
                        "requireResidentKey": false,
                        "userVerification": "preferred"
                    },
                    "extensions": {
                        "uvm": true,
                        "credProps": true
                    }
                }
            }"#,
            )
            .build();

        let update = app.resolve(request, response).expect("an update");

        // check that the app emitted a CreationChallenge event,
        let actual = update.events[0].clone();
        let ccr = assert_matches!(actual.clone(), Event::CreationChallenge(Ok(mut r)) => r.take_body().unwrap());
        assert_eq!(
            ccr.public_key.challenge.to_string(),
            "LnWGR_0kcTrx_qqFPQEZzfsogvic6bSLfXnihBzUYAg"
        );

        // push the event into the app
        let mut update = app.update(actual, &mut model);

        // check that the app emitted a CreateCredential effect,
        assert_let!(Effect::Passkey(request), &mut update.effects[0]);

        // check that the request is to create a credential
        let actual = &request.operation;
        let bytes = serde_json::to_vec(&ccr).unwrap();
        let expected = &PasskeyOperation::CreateCredential(bytes);
        assert_eq!(actual, expected);

        // simulate a successful response from the authenticator
        // let cred = RegisterPublicKeyCredential {
        //     id: "brs4tqNATYib4pRlF74jSg".to_string(),
        //     raw_id: Base64UrlSafeData(vec![1, 2, 3]),
        //     response: webauthn_rs_proto::AuthenticatorResponse::AuthenticatorAttestationResponse(
        //         webauthn_rs_proto::AttestationResponse {
        //             client_data_json: "client_data_json".to_string(),
        //             attestation_object: "attestation_object".to_string(),
        //         },
        //     ),
        //     type_: "public-key".to_string(),
        //     extensions: None,
        // };
        // let response = PasskeyOutput::RegisterCredential(vec![1, 2, 3]);
        // let update = app.resolve(request, response).expect("an update");
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

    // #[test]
    // fn login() {
    //     let app = AppTester::<App, _>::default();

    //     let mut model = Model::default();

    //     let event = Event::Login("stu".to_string());

    //     let mut update = app.update(event, &mut model);
    //     assert_effect!(update, Effect::Render(_));

    //     insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
    //     ---
    //     status:
    //       Info: "logging in \"stu\"..."
    //     "###);

    //     assert_let!(Effect::Passkey(request), &mut update.effects[1]);

    //     // let actual = &request.operation;
    //     // let expected = &PasskeyOperation::Login("stu".to_string());
    //     // assert_eq!(actual, expected);

    //     // // simulate a successful response from the server
    //     // let response = PasskeyOutput::LoggedIn;
    //     // let update = app.resolve(request, response).expect("an update");

    //     // let actual = update.events[0].clone();
    //     // assert_matches!(actual, Event::LoggedIn);

    //     // let update = app.update(actual, &mut model);

    //     // assert_effect!(update, Effect::Render(_));

    //     // insta::assert_yaml_snapshot!(app.view(&mut model), @r###"
    //     // ---
    //     // status:
    //     //   Info: "logged in \"stu\""
    //     // "###);
    // }

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
