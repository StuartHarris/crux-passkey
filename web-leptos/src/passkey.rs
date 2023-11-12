use anyhow::Result;
use leptos::*;
use shared::passkey::{PasskeyOperation, PasskeyOutput};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::JsFuture;
use webauthn_rs_proto::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

pub async fn request(operation: &PasskeyOperation) -> Result<PasskeyOutput> {
    match operation {
        PasskeyOperation::CreateCredential(bytes) => {
            let ccr = serde_json::from_slice::<CreationChallengeResponse>(bytes)?;
            // First, convert from our webauthn proto json safe format, into the browser
            // compatible struct, with everything decoded as needed.
            let c_options: web_sys::CredentialCreationOptions = ccr.into();

            // Create a promise that calls the browsers navigator.credentials.create api.
            let promise = window()
                .navigator()
                .credentials()
                .create_with_options(&c_options)
                .expect_throw("Unable to create promise");
            match JsFuture::from(promise).await {
                Ok(js_val) => {
                    let cred = web_sys::PublicKeyCredential::from(js_val);
                    let cred = RegisterPublicKeyCredential::from(cred);

                    Ok(PasskeyOutput::RegisterCredential(serde_json::to_vec(
                        &cred,
                    )?))
                }
                Err(e) => Ok(PasskeyOutput::Error(format!(
                    "Failed to create credential: {:?}",
                    e,
                ))),
            }
        }
        PasskeyOperation::RequestCredential(bytes) => {
            let ccr = serde_json::from_slice::<RequestChallengeResponse>(bytes)?;
            // First, convert from our webauthn proto json safe format, into the browser
            // compatible struct, with everything decoded as needed.
            let c_options: web_sys::CredentialRequestOptions = ccr.into();

            // Create a promise that calls the browsers navigator.credentials.create api.
            let promise = window()
                .navigator()
                .credentials()
                .get_with_options(&c_options)
                .expect_throw("Unable to create promise");
            match JsFuture::from(promise).await {
                Ok(js_val) => {
                    let cred = web_sys::PublicKeyCredential::from(js_val);
                    let cred = PublicKeyCredential::from(cred);

                    Ok(PasskeyOutput::Credential(serde_json::to_vec(&cred)?))
                }
                Err(e) => Ok(PasskeyOutput::Error(format!(
                    "Failed to authenticate: {:?}",
                    e,
                ))),
            }
        }
    }
}
