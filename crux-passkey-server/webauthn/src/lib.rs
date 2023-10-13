use anyhow::Result;
use serde::Serialize;
use spin_sdk::{
    http::{Request, Response},
    http_component,
    sqlite::{Connection, ValueParam},
};

#[http_component]
fn handle_request(_req: Request) -> Result<Response> {
    let connection = Connection::open_default()?;

    let execute_params = [
        ValueParam::Text("Try out Spin SQLite"),
        ValueParam::Text("Friday"),
    ];
    connection.execute(
        "INSERT INTO todos (description, due) VALUES (?, ?)",
        execute_params.as_slice(),
    )?;

    let row_set = connection.execute("SELECT id, description, due FROM todos", &[])?;

    let todos: Vec<_> = row_set
        .rows()
        .map(|row| ToDo {
            id: row.get::<u32>("id").unwrap(),
            description: row.get::<&str>("description").unwrap().to_owned(),
            due: row.get::<&str>("due").unwrap().to_owned(),
        })
        .collect();

    let body = serde_json::to_vec(&todos)?;
    Ok(http::Response::builder()
        .status(200)
        .body(Some(body.into()))?)
}

// Helper for returning the query results as JSON
#[derive(Serialize)]
struct ToDo {
    id: u32,
    description: String,
    due: String,
}
