use anyhow::Result;
use cookie::SameSite;
use spin_sdk::{
    http,
    sqlite::{Connection, ValueParam},
};
use std::{fmt::Display, str::FromStr};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SessionId(pub Uuid);

impl SessionId {
    fn from_request(req: &http::Request, cookie_name: &str) -> Result<Option<Self>> {
        let Some(cookie) = req.headers().get("cookie") else {
            return Ok(None);
        };

        let cookie = cookie.to_str()?;
        for cookie in cookie.split("; ") {
            let mut cookie = cookie.split("=");
            let Some(name) = cookie.next() else {
                continue;
            };
            let Some(value) = cookie.next() else {
                continue;
            };
            if name == cookie_name {
                return Ok(Some(SessionId(Uuid::from_str(value)?)));
            }
        }
        Ok(None)
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

    pub fn retrieve(req: &http::Request, cookie_name: &str) -> Result<Option<Self>> {
        let Some(session_id) = SessionId::from_request(&req, cookie_name)? else {
            return Ok(None);
        };

        let Some(session) = SqliteSessionStore::get(&session_id)? else {
            return Ok(None);
        };
        SqliteSessionStore::remove(&session_id)?;
        Ok(Some(session))
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

#[cfg(test)]
mod tests {
    #[test]
    fn test_get_cookie_from_headers() {
        use super::*;

        let mut req = http::Request::new(None);
        let headers = req.headers_mut();
        headers.append(
            "cookie",
            "crux-passkey.register=8a164194-c931-4127-a8c7-9a9d3ad60d7e; crux-passkey.login=21203bd1-1753-464f-9a3a-14e62d0ef0fa"
                .parse()
                .unwrap(),
        );
        assert_eq!(
            SessionId::from_request(&req, "crux-passkey.login").unwrap(),
            Some(SessionId(
                Uuid::parse_str("21203bd1-1753-464f-9a3a-14e62d0ef0fa").unwrap()
            ))
        );
        assert_eq!(
            SessionId::from_request(&req, "crux-passkey.register").unwrap(),
            Some(SessionId(
                Uuid::parse_str("8a164194-c931-4127-a8c7-9a9d3ad60d7e").unwrap()
            ))
        );
        assert_eq!(SessionId::from_request(&req, "blah").unwrap(), None);
    }
}
