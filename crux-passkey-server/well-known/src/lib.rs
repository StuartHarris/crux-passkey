use anyhow::Result;
use include_dir::{include_dir, Dir};
use spin_sdk::{
    http::{IntoResponse, Request, Response},
    http_component,
};

static PUBLIC_DIR: Dir = include_dir!("public");

#[http_component]
fn handle_well_known(req: Request) -> Result<impl IntoResponse> {
    let Some(path) = req.header("spin-path-info") else {
        return Err(anyhow::anyhow!("no path info"));
    };
    let Some(path) = path.as_str() else {
        return Err(anyhow::anyhow!("path info not a string"));
    };
    let path = path.trim_start_matches("/");
    println!("well-known: {}", path);
    match PUBLIC_DIR.get_file(path) {
        Some(f) => Ok(Response::builder()
            .status(200)
            .header("Content-Type", "application/json")
            .body(f.contents().to_vec())
            .build()),
        None => Ok(Response::new(404, "Not found")),
    }
}
