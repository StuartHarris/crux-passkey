use anyhow::{anyhow, Result};
use gloo_net::http;
use leptos::*;
use log::info;
use shared::passkey::{PasskeyOperation, PasskeyOutput};
use wasm_bindgen::UnwrapThrowExt;
use wasm_bindgen_futures::JsFuture;
use webauthn_rs_proto::{
    CreationChallengeResponse, PublicKeyCredential, RegisterPublicKeyCredential,
    RequestChallengeResponse,
};

pub async fn request(operation: &PasskeyOperation) -> Result<PasskeyOutput> {
    match operation {
        PasskeyOperation::Register(user_name) => {
            info!("Registering user: {}", user_name);
            register(&user_name).await?;
            Ok(PasskeyOutput::Registered)
        }
        PasskeyOperation::Login(user_name) => {
            info!("Logging in user: {}", user_name);
            login(&user_name).await?;
            Ok(PasskeyOutput::LoggedIn)
        }
    }
}

async fn register(user_name: &str) -> Result<()> {
    let start_url = &format!("/auth/register_start/{}", user_name);
    let finish_url = "/auth/register_finish";
    let response = http::Request::get(start_url).send().await?;
    if response.ok() {
        let ccr = response.json::<CreationChallengeResponse>().await?;

        // First, convert from our webauthn proto json safe format, into the browser
        // compatible struct, with everything decoded as needed.
        let c_options: web_sys::CredentialCreationOptions = ccr.into();
        info!("c_options: {:?}", c_options);

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

                let response = http::Request::post(finish_url)
                    .json(&cred)
                    .expect("Failed to serialize cred")
                    .send()
                    .await?;
                if response.ok() {
                    info!("Registered user: {}", user_name);
                    Ok(())
                } else {
                    Err(anyhow!("Failed to register: {}", response.status()))
                }
            }
            e @ Err(_) => Err(anyhow!("Failed to register: {:?}", e)),
        }
    } else {
        Err(anyhow!("Failed to register: {}", response.status()))
    }
}

async fn login(user_name: &str) -> Result<()> {
    let start_url = &format!("/auth/login_start/{}", user_name);
    let finish_url = "/auth/login_finish";
    let response = http::Request::get(start_url).send().await?;
    if response.ok() {
        let ccr = response.json::<RequestChallengeResponse>().await?;

        // First, convert from our webauthn proto json safe format, into the browser
        // compatible struct, with everything decoded as needed.
        let c_options: web_sys::CredentialRequestOptions = ccr.into();
        info!("c_options: {:?}", c_options);

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

                let response = http::Request::post(finish_url)
                    .json(&cred)
                    .expect("Failed to serialize cred")
                    .send()
                    .await?;
                if response.ok() {
                    info!("Logged in user: {}", user_name);
                    Ok(())
                } else {
                    Err(anyhow!("Failed to authenticate: {}", response.status()))
                }
            }
            e @ Err(_) => Err(anyhow!("Failed to authenticate: {:?}", e)),
        }
    } else {
        Err(anyhow!("Failed to authenticate: {}", response.status()))
    }
}
