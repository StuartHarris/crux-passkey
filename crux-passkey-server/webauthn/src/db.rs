use anyhow::Result;
use spin_sdk::sqlite::{self, Connection, ValueParam};
use uuid::Uuid;
use webauthn_rs::prelude::{Base64UrlSafeData, Passkey};

pub(crate) fn get_user_credential_ids(
    connection: &Connection,
    user_unique_id: Uuid,
) -> Result<Vec<Base64UrlSafeData>> {
    Ok(get_user_credentials(connection, &user_unique_id)?
        .iter()
        .map(|passkey| passkey.cred_id().clone())
        .collect())
}

pub(crate) fn get_user_credentials(
    connection: &Connection,
    user_unique_id: &Uuid,
) -> Result<Vec<Passkey>> {
    Ok(connection
        .execute(
            "SELECT credentials FROM credentials WHERE user_id = ?1",
            &[ValueParam::Blob(user_unique_id.as_bytes())],
        )?
        .rows()
        .filter_map(|row| {
            row.get::<&[u8]>("credentials").and_then(|bytes| {
                Some(serde_json::from_slice::<Passkey>(bytes).expect("valid passkey"))
            })
        })
        .collect())
}

pub(crate) fn get_user_unique_id(connection: &Connection, username: &str) -> Result<Uuid> {
    Ok(connection
        .execute(
            "SELECT user_id FROM user WHERE user_name = ?1",
            &[ValueParam::Text(username)],
        )?
        .rows()
        .next()
        .and_then(|row| row.get::<&[u8]>("user_id"))
        .and_then(|bytes| Some(Uuid::from_slice(bytes).expect("valid UUID")))
        .unwrap_or_else(Uuid::new_v4))
}

pub(crate) fn save_credentials(
    connection: &Connection,
    user_unique_id: &Uuid,
    passkey: &Passkey,
) -> Result<sqlite::QueryResult> {
    Ok(connection.execute(
        "INSERT INTO credentials (user_id, credentials) VALUES (?1, ?2)",
        &[
            ValueParam::Blob(user_unique_id.as_bytes()),
            ValueParam::Blob(&serde_json::to_vec(&passkey)?),
        ],
    )?)
}

pub(crate) fn save_user(
    connection: &Connection,
    username: &str,
    user_unique_id: &Uuid,
) -> Result<sqlite::QueryResult> {
    Ok(connection.execute(
        "INSERT INTO user (user_name, user_id) VALUES (?1, ?2)",
        &[
            ValueParam::Text(&username),
            ValueParam::Blob(user_unique_id.as_bytes()),
        ],
    )?)
}
