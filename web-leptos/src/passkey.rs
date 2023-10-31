use anyhow::Result;
use log::info;
use shared::passkey::{PasskeyOperation, PasskeyOutput};

pub async fn request(operation: &PasskeyOperation) -> Result<PasskeyOutput> {
    match operation {
        PasskeyOperation::Register(user_name) => {
            info!("Registering user: {}", user_name);
            Ok(PasskeyOutput::Registered)
        }
        PasskeyOperation::Login(user_name) => {
            info!("Logging in user: {}", user_name);
            Ok(PasskeyOutput::LoggedIn)
        }
    }
}
