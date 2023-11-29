mod auth;
mod db;
mod session;

use anyhow::Result;
use spin_sdk::{
    http::{IntoResponse, Request, Response, Router},
    http_component,
};

#[http_component]
fn handle_request(req: Request) -> Result<impl IntoResponse> {
    let mut router = Router::new();
    router.get("/auth/register_start/:username", auth::register_start);
    router.post("/auth/register_finish", auth::register_finish);
    router.get("/auth/login_start/:username", auth::login_start);
    router.post("/auth/login_finish", auth::login_finish);
    router.any("/*", |_: Request, _| Response::new(404, "Not found"));
    Ok(router.handle(req))
}
