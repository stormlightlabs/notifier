//! mod server defines the async listener creator and an application
//! that has a root router for checking the status of the web services
//! as well as the webhook route.
//!
//! TODO: Configure HTTPS and update tunnel with `tls_rustls`
//! TODO: logging middleware
use crate::helpers;
use axum::{
    body::{Body, Bytes},
    extract::{Request, State},
    http::{status::StatusCode, HeaderMap, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Form, Json, Router,
};
use http_body_util::BodyExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{slice::Iter, sync::Arc};
use tokio::sync::mpsc;

pub struct SharedState {
    pub event_sender: mpsc::Sender<Value>,
    pub secrets: helpers::Secrets,
}

enum GithubHeaders {
    HookID,
    Event,
    Deliver,
    Signature,
    Signature256,
    UserAgent,
    HookInstallationTargetType,
    HookInstallationTargetID,
}

impl GithubHeaders {
    fn as_str(&self) -> &'static str {
        match self {
            GithubHeaders::HookID => "X-GitHub-Hook-ID",
            GithubHeaders::Event => "X-GitHub-Event",
            GithubHeaders::Deliver => "X-Github-Delivery",
            GithubHeaders::Signature => "X-Hub-Signature",
            GithubHeaders::Signature256 => "X-Hub-Signature-256",
            GithubHeaders::UserAgent => "User-Agent",
            GithubHeaders::HookInstallationTargetType => "X-Github-Hook-Installation-Target-Type",
            GithubHeaders::HookInstallationTargetID => "X-Github-Hook-Installation-Target-ID",
        }
    }

    pub fn iterator() -> Iter<'static, GithubHeaders> {
        static HEADERS: [GithubHeaders; 8] = [
            GithubHeaders::HookID,
            GithubHeaders::Event,
            GithubHeaders::Deliver,
            GithubHeaders::Signature,
            GithubHeaders::Signature256,
            GithubHeaders::UserAgent,
            GithubHeaders::HookInstallationTargetType,
            GithubHeaders::HookInstallationTargetID,
        ];

        HEADERS.iter()
    }
}

#[derive(Serialize, Deserialize)]
struct Status {
    discord: bool,
    github: bool,
}

/// TODO: Implement handlers for each service
fn check_status() -> Status {
    Status {
        discord: true,
        github: true,
    }
}

async fn root() -> Json<Status> {
    let status = check_status();

    Json(status)
}

fn handle_user_agent_value(val: &HeaderValue) -> Result<(), ()> {
    match val.to_str() {
        Ok(value) => {
            if value.contains("GitHub-Hookshot/") {
                return Ok(());
            }

            Err(())
        }
        Err(_) => Err(()),
    }
}

fn handle_signature_256(val: &HeaderValue, _webhook_secret: &String) -> Result<(), ()> {
    match val.to_str() {
        Ok(_) => Ok(()),
        Err(_) => Err(()),
    }
}

/// Check the x-github-event header to learn what event type was sent.
/// X-Hub-Signature-256 (note: requires webhook secret to be added)
///
/// Github sends a POST request
///
/// Return a 202, Accepted (within 10s)
///
/// TODO: Check that the event and actions sent are among the set of
/// subscriptions
pub async fn github_webhook_handler(
    State(state): State<Arc<SharedState>>,
    header_map: HeaderMap,
    Form(payload): Form<Value>,
) -> impl IntoResponse {
    let headers = header_map.clone();

    if headers.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    match handle_headers(&headers, &state.secrets.webhook_secret) {
        Ok(_) => {
            if let Err(_) = state.event_sender.send(payload).await {
                return StatusCode::INTERNAL_SERVER_ERROR;
            }

            StatusCode::ACCEPTED
        }
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

fn handle_headers(headers: &HeaderMap, webhook_secret: &String) -> Result<(), ()> {
    for h in GithubHeaders::iterator() {
        match headers.get(h.as_str()) {
            Some(value) => match h {
                GithubHeaders::UserAgent => {
                    let Ok(_) = handle_user_agent_value(value) else {
                        return Ok(());
                    };
                }
                GithubHeaders::Signature256 => {
                    let Ok(_) = handle_signature_256(value, &webhook_secret) else {
                        return Err(());
                    };
                }
                GithubHeaders::Event
                | GithubHeaders::Deliver
                | GithubHeaders::HookInstallationTargetID
                | GithubHeaders::HookInstallationTargetType => {
                    println!("{}: '{value:?}'", h.as_str())
                }
                _ => continue,
            },
            None => return Err(()),
        }
    }

    Ok(())
}

pub fn create_service(state: Arc<SharedState>) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/gh", post(github_webhook_handler))
        .layer(axum::middleware::from_fn(print_request_response))
        .with_state(state)
}

pub async fn create_listener(port: u16) -> tokio::net::TcpListener {
    let addr = format!("0.0.0.0:{}", port);
    tokio::net::TcpListener::bind(addr).await.unwrap()
}

async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let res = Response::from_parts(parts, Body::from(bytes));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
