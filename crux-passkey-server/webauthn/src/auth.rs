use crate::{
    db,
    session::{Session, SessionStore, SqliteSessionStore},
};
use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Params, Request, Response},
    sqlite::Connection,
    variables,
};

use uuid::Uuid;
use webauthn_rs::{
    prelude::{
        CredentialID, PasskeyAuthentication, PasskeyRegistration, PublicKeyCredential,
        RegisterPublicKeyCredential, Url,
    },
    Webauthn, WebauthnBuilder,
};

const LOGIN_COOKIE: &str = "crux-passkey.login";
const REGISTER_COOKIE: &str = "crux-passkey.register";

fn webauthn() -> Result<Webauthn> {
    let rp_id = variables::get("rp_id")?;
    let rp_origin = Url::parse(&format!("https://{rp_id}"))?;
    let webauthn = WebauthnBuilder::new(&rp_id, &rp_origin)?
        .rp_name("Crux Passkey")
        .build()?;
    Ok(webauthn)
}

pub(crate) fn register_start(_req: Request, params: Params) -> Result<impl IntoResponse> {
    println!("register_start");

    let Some(username) = params.get("username") else {
        return Ok(Response::new(400, "no username"));
    };

    println!("Registering user: {}", username);

    let connection = Connection::open_default()?;
    println!("db connection opened");

    let user_unique_id = db::get_user_unique_id(&connection, username)?;
    println!("user_unique_id: {:?}", user_unique_id);

    let exclude_credentials: Vec<CredentialID> =
        db::get_user_credential_ids(&connection, user_unique_id)?;
    println!("exclude_credentials: {:?}", exclude_credentials);

    match webauthn()?.start_passkey_registration(
        user_unique_id,
        &username,
        &username,
        Some(exclude_credentials),
    ) {
        Ok((ccr, reg_state)) => {
            let mut session = Session::new();
            session.data = serde_json::to_vec(&(username, user_unique_id, reg_state))?;
            SqliteSessionStore::set(&session)?;
            let body = serde_json::to_string(&ccr)?;
            let cookie = session.cookie(REGISTER_COOKIE, "/auth");
            Ok(Response::builder()
                .status(200)
                .header("Content-Type", "application/json")
                .header("set-cookie", cookie)
                .body(body)
                .build())
        }
        Err(e) => {
            println!("register_start: {:?}", e);
            Ok(Response::new(400, format!("error {:?}", e)))
        }
    }
}

pub(crate) fn register_finish(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let Some(session) = Session::retrieve(&req, REGISTER_COOKIE)? else {
        return Ok(Response::new(400, "no session"));
    };

    let reg: RegisterPublicKeyCredential = serde_json::from_slice(req.body())?;

    let (username, user_unique_id, reg_state): (String, Uuid, PasskeyRegistration) =
        serde_json::from_slice(session.data.as_slice())?;

    match webauthn()?.finish_passkey_registration(&reg, &reg_state) {
        Ok(passkey) => {
            let connection = Connection::open_default()?;
            db::save_user(&connection, &username, &user_unique_id)?;
            db::save_credentials(&connection, &user_unique_id, &passkey)?;
            println!("Registration Successful!");
            Ok(Response::builder().status(200).build())
        }
        Err(e) => {
            println!("register_finish: {:?}", e);
            Err(e.into())
        }
    }
}

pub(crate) fn login_start(_req: Request, params: Params) -> Result<impl IntoResponse> {
    let Some(username) = params.get("username") else {
        return Ok(Response::new(400, "no username"));
    };

    let connection = Connection::open_default()?;
    let user_unique_id = db::get_user_unique_id(&connection, username)?;
    let credentials = db::get_user_credentials(&connection, &user_unique_id)?;
    if credentials.len() == 0 {
        return Ok(Response::new(400, "no credentials found"));
    }

    match webauthn()?.start_passkey_authentication(credentials.as_slice()) {
        Ok((rcr, auth_state)) => {
            let mut session = Session::new();
            session.data = serde_json::to_vec(&(user_unique_id, auth_state))?;
            SqliteSessionStore::set(&session)?;
            let cookie = session.cookie(LOGIN_COOKIE, "/auth");
            let body = serde_json::to_string(&rcr)?;
            Ok(Response::builder()
                .header("Content-Type", "application/json")
                .header("set-cookie", cookie)
                .status(200)
                .body(body)
                .build())
        }
        Err(e) => {
            println!("login_start: {:?}", e);
            Err(e.into())
        }
    }
}

pub(crate) fn login_finish(req: Request, _params: Params) -> Result<impl IntoResponse> {
    let Some(session) = Session::retrieve(&req, LOGIN_COOKIE)? else {
        return Ok(Response::new(400, "no session"));
    };

    let auth: PublicKeyCredential = serde_json::from_slice(req.body())?;

    let (user_unique_id, auth_state): (Uuid, PasskeyAuthentication) =
        serde_json::from_slice(session.data.as_slice())?;

    match webauthn()?.finish_passkey_authentication(&auth, &auth_state) {
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
            Ok(Response::builder().status(200).build())
        }
        Err(e) => {
            println!("login_finish: {:?}", e);
            Err(e.into())
        }
    }
}
