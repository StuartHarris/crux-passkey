use std::fmt::Display;

use anyhow::Result;
use spin_sdk::sqlite::{Connection, ValueParam};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct SessionId(pub Uuid);

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
}

pub trait SessionStore {
    fn get(id: &SessionId) -> Result<Option<Session>>;
    fn set(session: Session) -> Result<()>;
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
        if let Some(row) = rows.next() {
            if let Some(data) = row.get::<&[u8]>("data") {
                Ok(Some(Session {
                    id: id.clone(),
                    data: data.to_vec(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    fn set(session: Session) -> Result<()> {
        let connection = Connection::open_default()?;
        let execute_params = [
            ValueParam::Blob(session.id.0.as_bytes()),
            ValueParam::Blob(session.data.as_slice()),
        ];
        connection.execute(
            "INSERT INTO user_session (id, data) VALUES (?1, ?2)",
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
