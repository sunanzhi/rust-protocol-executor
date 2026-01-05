use std::collections::HashMap;
use serde::Deserialize;
use crate::executor::protocol::{AssertDTO, ParamDTO};

#[derive(Debug, Deserialize)]
pub struct HttpProtocolModel {
    request: HttpRequest,
    response: HttpResponse,
    options: HttpOptions,
}

#[derive(Debug, Deserialize)]
pub struct HttpRequest {
    url: String,
    method: String,
    headers: HashMap<String, String>,
    query_params: HashMap<String, String>,
    body: HttpRequestBodyDTO,
}

#[derive(Debug, Deserialize)]
pub struct HttpRequestBodyDTO {
    mode: HttpRequestBodyModeEnum,
    json: String,
    form: HashMap<String, ParamDTO>,
    raw: String,
}

#[derive(Debug, Deserialize)]
struct HttpResponse {
    status: u64,
    headers: HashMap<String, String>,
    body_asserts: HashMap<String, AssertDTO>,
}

#[derive(Debug, Deserialize)]
struct HttpOptions {
    http_version: Option<String>,
    timeout_ms: Option<u64>,
    retry: HttpRetry,
    follow_redirects: bool,
    verify_ssl: bool,
}

#[derive(Debug, Deserialize)]
struct HttpRetry {
    count: u64,
    interval_ms: u64,
}

#[derive(Debug, Deserialize)]
enum HttpRequestBodyModeEnum {
    Json,
    FormUrlencoded,
    Multipart,
    Raw,
    Binary,
    None
}