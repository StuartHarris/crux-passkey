mod session;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use session::{Session, SessionId, SessionStore, SqliteSessionStore};
use spin_sdk::{
    http::{Params, Request, Response, Router},
    http_component,
};

#[http_component]
fn handle_request(req: Request) -> Result<Response> {
    let mut router = Router::new();
    router.get("/auth/register_start/:username", register_start);
    router.get("/auth/register_finish", register_finish);
    router.post("/auth/login_start/:username", login_start);
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
struct ToDo {
    id: u32,
    description: String,
    due: String,
}

fn register_start(_req: Request, _params: Params) -> Result<Response> {
    let mut session = Session::new();
    session.data = serde_json::to_vec(&ToDo {
        id: 1,
        description: "Do the thing".to_string(),
        due: "2021-01-01".to_string(),
    })?;
    SqliteSessionStore::set(&session)?;

    Ok(http::Response::builder()
        .header("set-cookie", session.cookie("session_id", "/auth"))
        .status(200)
        .body(Some("yay!".into()))?)
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

    let todo: ToDo = serde_json::from_slice(&session.data)?;
    Ok(http::Response::builder()
        .status(200)
        .body(Some(format!("{:?}", todo).into()))?)
}

fn login_start(_req: Request, _params: Params) -> Result<Response> {
    todo!();
}

fn login_finish(_req: Request, _params: Params) -> Result<Response> {
    todo!();
}
