use std::str;

use anyhow::Result;
use gloo_net::http;

use log::info;
use shared::http::protocol::{HttpRequest, HttpResponse};
use web_sys::wasm_bindgen::JsValue;

pub async fn request(
    HttpRequest {
        method,
        url,
        headers,
        body,
    }: &HttpRequest,
) -> Result<HttpResponse> {
    info!("url: {}", url);
    let mut request = match method.as_str() {
        "GET" => http::Request::get(url),
        "POST" => http::Request::post(url),
        _ => panic!("not yet handling this method"),
    };

    for header in headers {
        request = request.header(&header.name, &header.value);
    }

    let response = if body.len() > 0 {
        request
            .body(JsValue::from_str(str::from_utf8(body)?))
            .expect("Failed to serialize body")
            .send()
            .await?
    } else {
        request.send().await?
    };

    let body = response.binary().await?;

    Ok(HttpResponse::status(response.status()).body(body).build())
}
