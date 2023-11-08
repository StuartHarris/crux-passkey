mod auth;
mod db;
mod session;

use spin_sdk::{
    http::{Request, Response, Router},
    http_component,
};

#[http_component]
fn handle_request(req: Request) -> Response {
    let mut router = Router::new();
    router.get("/auth/register_start/:username", auth::register_start);
    router.post("/auth/register_finish", auth::register_finish);
    router.get("/auth/login_start/:username", auth::login_start);
    router.post("/auth/login_finish", auth::login_finish);
    router.any("/*", |_: Request, _| bad_request("not found"));
    router.handle(req)
}

fn bad_request(reason: &str) -> Response {
    Response::builder()
        .status(400)
        .body(reason.to_string())
        .build()
}
