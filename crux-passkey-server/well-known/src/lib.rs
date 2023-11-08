use anyhow::Result;
use include_dir::{include_dir, Dir};
use spin_sdk::{
    http::{Request, Response},
    http_component,
};

static PUBLIC_DIR: Dir = include_dir!("public");

#[http_component]
fn handle_well_known(req: Request) -> Result<Response> {
    let path = req
        .headers()
        .get("spin-path-info")
        .expect("path info header")
        .to_str()
        .expect("header is ascii")
        .trim_start_matches("/");
    println!("well-known: {}", path);
    match PUBLIC_DIR.get_file(path) {
        Some(f) => Ok(http::Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(Some(f.contents().to_vec().into()))?),
        None => Ok(http::Response::builder()
            .status(404)
            .body(Some("Not found".into()))?),
    }
}
