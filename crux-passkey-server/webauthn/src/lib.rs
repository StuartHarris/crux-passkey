mod session;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use session::{Session, SessionId, SessionStore, SqliteSessionStore};
use spin_sdk::{
    http::{Params, Request, Response, Router},
    http_component,
    sqlite::{Connection, ValueParam},
};
use url::Url;
use uuid::Uuid;
use webauthn_rs::{
    prelude::{CredentialID, Passkey},
    WebauthnBuilder,
};

#[http_component]
fn handle_request(req: Request) -> Result<Response> {
    let mut router = Router::new();
    router.get("/auth/register_start/:username", register_start);
    router.post("/auth/register_finish", register_finish);
    router.get("/auth/login_start/:username", login_start);
    router.post("/auth/login_finish", login_finish);
    router.any("/*", |_, _| {
        Ok(http::Response::builder()
            .status(404)
            .body(Some("Not found".into()))?)
    });
    router.handle(req)
}

// Helper for returning the query results as JSON
#[derive(Serialize, Deserialize, Debug)]
struct User {
    id: u128,
    name: String,
}

fn register_start(_req: Request, params: Params) -> Result<Response> {
    // find user's id from their username
    let connection = Connection::open_default()?;
    let Some(username) = params.get("username") else {
        return bad_request("no username");
    };
    let execute_params = [ValueParam::Text(username)];
    let row_set = connection.execute(
        "SELECT user_id FROM user WHERE user_name = ?1",
        &execute_params,
    )?;
    let mut rows = row_set.rows();
    let user_unique_id = rows
        .next()
        .and_then(|row| row.get::<&[u8]>("user_id"))
        .and_then(|bytes| Some(Uuid::from_slice(bytes).expect("valid UUID")))
        .unwrap_or_else(Uuid::new_v4);

    let Some(username) = params.get("username") else {
        return bad_request("no username");
    };
    let execute_params = [ValueParam::Blob(user_unique_id.as_bytes())];
    let row_set = connection.execute(
        "SELECT credentials FROM credentials WHERE user_id = ?1",
        &execute_params,
    )?;
    let exclude_credentials: Vec<CredentialID> = row_set
        .rows()
        .filter_map(|row| {
            row.get::<&[u8]>("credentials")
                .and_then(|bytes| {
                    Some(serde_json::from_slice::<Passkey>(bytes).expect("valid passkey"))
                })
                .map(|passkey| passkey.cred_id().clone())
        })
        .collect();

    // Effective domain name.
    let rp_id = "localhost";
    let rp_origin = Url::parse("https://localhost:3000").expect("Invalid URL");
    let builder = WebauthnBuilder::new(rp_id, &rp_origin).expect("Invalid configuration");

    let builder = builder.rp_name("Spin Webauthn-rs");

    let webauthn = builder.build().expect("Invalid configuration");

    match webauthn.start_passkey_registration(
        user_unique_id,
        &username,
        &username,
        Some(exclude_credentials),
    ) {
        Ok((ccr, reg_state)) => {
            let mut session = Session::new();
            session.data = serde_json::to_vec(&reg_state)?;
            SqliteSessionStore::set(&session)?;
            println!("Registration Successful!");
            Ok(http::Response::builder()
                .header("Content-Type", "application/json")
                .header(
                    "set-cookie",
                    session.cookie("crux-passkey.register", "/auth"),
                )
                .status(200)
                .body(Some(
                    serde_json::to_string(&ccr)
                        .expect("serialize passkey")
                        .into(),
                ))?)
        }
        Err(e) => {
            println!("challenge_register -> {:?}", e);
            return Err(e.into());
        }
    }
}

fn register_finish(req: Request, _params: Params) -> Result<Response> {
    let Some(session_id) = SessionId::from_request(&req)? else {
        return Ok(http::Response::builder()
            .status(400)
            .body(Some("no session cookie".into()))?);
    };

    let Some(session) = SqliteSessionStore::get(&session_id)? else {
        return Ok(http::Response::builder()
            .status(400)
            .body(Some("no session".into()))?);
    };

    Ok(http::Response::builder()
        .status(200)
        .body(Some(format!("{:?}", session).into()))?)
}

fn login_start(_req: Request, _params: Params) -> Result<Response> {
    todo!();
}

fn login_finish(_req: Request, _params: Params) -> Result<Response> {
    todo!();
}

fn bad_request(reason: &str) -> Result<Response> {
    Ok(http::Response::builder()
        .status(400)
        .body(Some(reason.to_string().into()))?)
}
