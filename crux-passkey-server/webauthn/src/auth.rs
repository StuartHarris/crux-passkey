use super::bad_request;
use crate::{
    db,
    session::{Session, SessionStore, SqliteSessionStore},
};
use anyhow::Result;
use spin_sdk::{
    http::{Params, Request, Response},
    sqlite::Connection,
};
use url::Url;
use uuid::Uuid;
use webauthn_rs::{
    self,
    prelude::{
        CredentialID, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
        RegisterPublicKeyCredential,
    },
    WebauthnBuilder,
};
const LOGIN_COOKIE: &str = "crux-passkey.login";
const REGISTER_COOKIE: &str = "crux-passkey.register";
const DOMAIN: &str = "crux-passkey-server-yrx9iojr.fermyon.app";

fn webauthn() -> webauthn_rs::Webauthn {
    let rp_id = DOMAIN;
    let rp_origin = Url::parse(&format!("https://{rp_id}")).expect("valid URL");
    let webauthn = WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid configuration")
        .rp_name("Spin Webauthn-rs")
        .build()
        .expect("Invalid configuration");
    webauthn
}

pub(crate) fn register_start(_req: Request, params: Params) -> Result<Response> {
    let Some(username) = params.get("username") else {
        return bad_request("no username");
    };

    let connection = Connection::open_default()?;
    let user_unique_id = db::get_user_unique_id(&connection, username)?;
    let exclude_credentials: Vec<CredentialID> =
        db::get_user_credential_ids(&connection, user_unique_id)?;

    match webauthn().start_passkey_registration(
        user_unique_id,
        &username,
        &username,
        Some(exclude_credentials),
    ) {
        Ok((ccr, reg_state)) => {
            let mut session = Session::new();
            session.data = serde_json::to_vec(&(username, user_unique_id, reg_state))?;
            SqliteSessionStore::set(&session)?;
            Ok(http::Response::builder()
                .header("Content-Type", "application/json")
                .header("set-cookie", session.cookie(REGISTER_COOKIE, "/auth"))
                .status(200)
                .body(Some(serde_json::to_string(&ccr)?.into()))?)
        }
        Err(e) => {
            println!("register_start: {:?}", e);
            return Err(e.into());
        }
    }
}

pub(crate) fn register_finish(req: Request, _params: Params) -> Result<Response> {
    let Some(session) = Session::retrieve(&req, REGISTER_COOKIE)? else {
        return bad_request("no session");
    };

    let Some(req) = req.body() else {
        return bad_request("no body");
    };

    let reg: RegisterPublicKeyCredential = serde_json::from_slice(req)?;

    let (username, user_unique_id, reg_state): (String, Uuid, PasskeyRegistration) =
        serde_json::from_slice(session.data.as_slice())?;

    match webauthn().finish_passkey_registration(&reg, &reg_state) {
        Ok(passkey) => {
            let connection = Connection::open_default()?;
            db::save_user(&connection, &username, &user_unique_id)?;
            db::save_credentials(&connection, &user_unique_id, &passkey)?;
            println!("Registration Successful!");
            Ok(http::Response::builder().status(200).body(None)?)
        }
        Err(e) => {
            println!("register_finish: {:?}", e);
            Err(e.into())
        }
    }
}

pub(crate) fn login_start(_req: Request, params: Params) -> Result<Response> {
    let Some(username) = params.get("username") else {
        return bad_request("no username");
    };

    let connection = Connection::open_default()?;
    let user_unique_id = db::get_user_unique_id(&connection, username)?;
    let credentials = db::get_user_credentials(&connection, &user_unique_id)?;
    if credentials.len() == 0 {
        return bad_request("no credentials found");
    }

    match webauthn().start_passkey_authentication(credentials.as_slice()) {
        Ok((rcr, auth_state)) => {
            let mut session = Session::new();
            session.data = serde_json::to_vec(&(user_unique_id, auth_state))?;
            SqliteSessionStore::set(&session)?;
            Ok(http::Response::builder()
                .header("Content-Type", "application/json")
                .header("set-cookie", session.cookie(LOGIN_COOKIE, "/auth"))
                .status(200)
                .body(Some(serde_json::to_string(&rcr)?.into()))?)
        }
        Err(e) => {
            println!("login_start: {:?}", e);
            Err(e.into())
        }
    }
}

pub(crate) fn login_finish(req: Request, _params: Params) -> Result<Response> {
    let Some(session) = Session::retrieve(&req, LOGIN_COOKIE)? else {
        return bad_request("no session");
    };

    let Some(req) = req.body() else {
        return bad_request("no body");
    };

    let auth: PublicKeyCredential = serde_json::from_slice(req)?;

    let (user_unique_id, auth_state): (Uuid, PasskeyAuthentication) =
        serde_json::from_slice(session.data.as_slice())?;

    match webauthn().finish_passkey_authentication(&auth, &auth_state) {
        Ok(auth_result) => {
            let connection = Connection::open_default()?;
            for cred in db::get_user_credentials(&connection, &user_unique_id)?
                .into_iter()
                .filter_map(|mut cred| {
                    cred.update_credential(&auth_result)
                        .map(|updated| updated.then(|| cred))
                })
                .flatten()
            {
                db::save_credentials(&connection, &user_unique_id, &cred)?;
            }
            println!("Login Successful!");
            Ok(http::Response::builder().status(200).body(None)?)
        }
        Err(e) => {
            println!("login_finish: {:?}", e);
            Err(e.into())
        }
    }
}
