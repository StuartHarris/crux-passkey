mod auth;
mod db;
mod session;

use anyhow::Result;
use spin_sdk::{
    http::{Request, Response, Router},
    http_component,
};

#[http_component]
fn handle_request(req: Request) -> Result<Response> {
    let mut router = Router::new();
    router.get("/auth/register_start/:username", auth::register_start);
    router.post("/auth/register_finish", auth::register_finish);
    router.get("/auth/login_start/:username", auth::login_start);
    router.post("/auth/login_finish", auth::login_finish);
    router.any("/*", |_, _| {
        Ok(http::Response::builder()
            .status(404)
            .body(Some("Not found".into()))?)
    });
    router.handle(req)
}

fn bad_request(reason: &str) -> Result<Response> {
    Ok(http::Response::builder()
        .status(400)
        .body(Some(reason.to_string().into()))?)
}
