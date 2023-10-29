mod session;

use anyhow::{anyhow, Result};
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
    prelude::{
        Base64UrlSafeData, CredentialID, Passkey, PasskeyRegistration, RegisterPublicKeyCredential,
    },
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
    let Some(username) = params.get("username") else {
        return bad_request("no username");
    };

    let connection = Connection::open_default()?;
    let user_unique_id = user_unique_id(&connection, username)?;
    let exclude_credentials: Vec<CredentialID> = exclude_credentials(&connection, user_unique_id)?;

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

fn webauthn() -> webauthn_rs::Webauthn {
    let rp_id = "localhost";
    let rp_origin = Url::parse(&format!("https://{rp_id}:3000")).expect("valid URL");
    let webauthn = WebauthnBuilder::new(rp_id, &rp_origin)
        .expect("Invalid configuration")
        .rp_name("Spin Webauthn-rs")
        .build()
        .expect("Invalid configuration");
    webauthn
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
    SqliteSessionStore::remove(&session_id)?;

    let req = req.body().as_deref().ok_or_else(|| anyhow!("no body"))?;
    let reg = serde_json::from_slice::<RegisterPublicKeyCredential>(req)?;

    let (username, user_unique_id, reg_state): (String, Uuid, PasskeyRegistration) =
        serde_json::from_slice(session.data.as_slice())?;

    match webauthn().finish_passkey_registration(&reg, &reg_state) {
        Ok(passkey) => {
            let connection = Connection::open_default()?;
            connection.execute(
                "INSERT INTO user (user_name, user_id) VALUES (?1, ?2)",
                &[
                    ValueParam::Text(&username),
                    ValueParam::Blob(user_unique_id.as_bytes()),
                ],
            )?;
            connection.execute(
                "INSERT INTO credentials (user_id, credentials) VALUES (?1, ?2)",
                &[
                    ValueParam::Blob(user_unique_id.as_bytes()),
                    ValueParam::Blob(&serde_json::to_vec(&passkey)?),
                ],
            )?;
            println!("Registration Successful!");
        }
        Err(e) => {
            println!("challenge_register -> {:?}", e);
            return Err(e.into());
        }
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

fn exclude_credentials(
    connection: &Connection,
    user_unique_id: Uuid,
) -> Result<Vec<Base64UrlSafeData>> {
    Ok(connection
        .execute(
            "SELECT credentials FROM credentials WHERE user_id = ?1",
            &[ValueParam::Blob(user_unique_id.as_bytes())],
        )?
        .rows()
        .filter_map(|row| {
            row.get::<&[u8]>("credentials")
                .and_then(|bytes| {
                    Some(serde_json::from_slice::<Passkey>(bytes).expect("valid passkey"))
                })
                .map(|passkey| passkey.cred_id().clone())
        })
        .collect())
}

fn user_unique_id(connection: &Connection, username: &str) -> Result<Uuid> {
    Ok(connection
        .execute(
            "SELECT user_id FROM user WHERE user_name = ?1",
            &[ValueParam::Text(username)],
        )?
        .rows()
        .next()
        .and_then(|row| row.get::<&[u8]>("user_id"))
        .and_then(|bytes| Some(Uuid::from_slice(bytes).expect("valid UUID")))
        .unwrap_or_else(Uuid::new_v4))
}
