use std::{fmt::Display, str::FromStr};

use anyhow::Result;
use cookie::SameSite;
use spin_sdk::sqlite::{Connection, ValueParam};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SessionId(pub Uuid);

impl SessionId {
    pub fn from_request(req: &spin_sdk::http::Request) -> Result<Option<Self>> {
        if let Some(cookie) = req.headers().get("cookie") {
            let cookie = cookie::Cookie::parse(std::str::from_utf8(cookie.as_bytes())?)?;
            let value = cookie.name_value().1;
            let session_id = SessionId(Uuid::from_str(value)?);
            Ok(Some(session_id))
        } else {
            Ok(None)
        }
    }
}

impl Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Session {
    pub id: SessionId,
    pub data: Vec<u8>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            id: SessionId(Uuid::new_v4()),
            data: vec![],
        }
    }

    pub fn cookie(&self, name: &str, path: &str) -> String {
        cookie::Cookie::build((name, self.id.to_string()))
            .path(path)
            .secure(true)
            .same_site(SameSite::Strict)
            .http_only(true)
            .to_string()
    }
}

pub trait SessionStore {
    fn get(id: &SessionId) -> Result<Option<Session>>;
    fn set(session: &Session) -> Result<()>;
    fn remove(id: &SessionId) -> Result<()>;
}

pub struct SqliteSessionStore {}

impl SessionStore for SqliteSessionStore {
    fn get(id: &SessionId) -> Result<Option<Session>> {
        let connection = Connection::open_default()?;
        let execute_params = [ValueParam::Blob(id.0.as_bytes())];
        let row_set = connection.execute(
            "SELECT data FROM user_session WHERE id = ?1",
            execute_params.as_slice(),
        )?;
        let mut rows = row_set.rows();
        let Some(row) = rows.next() else {
            return Ok(None);
        };
        let Some(data) = row.get::<&[u8]>("data") else {
            return Ok(None);
        };
        Ok(Some(Session {
            id: id.clone(),
            data: data.to_vec(),
        }))
    }

    fn set(session: &Session) -> Result<()> {
        let connection = Connection::open_default()?;
        let execute_params = [
            ValueParam::Blob(session.id.0.as_bytes()),
            ValueParam::Blob(session.data.as_slice()),
        ];
        connection.execute(
            "INSERT OR REPLACE INTO user_session (id, data) VALUES (?1, ?2)",
            execute_params.as_slice(),
        )?;
        Ok(())
    }

    fn remove(id: &SessionId) -> Result<()> {
        let connection = Connection::open_default()?;
        let execute_params = [ValueParam::Blob(id.0.as_bytes())];
        connection.execute(
            "DELETE FROM user_session WHERE id = ?1",
            execute_params.as_slice(),
        )?;
        Ok(())
    }
}
