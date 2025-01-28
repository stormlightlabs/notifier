//! mod server defines the async listener creator and an application
//! that has a root router for checking the status of the web services
//! as well as the webhook route.
//!
//! TODO: Configure HTTPS and update tunnel with `tls_rustls`
//! TODO: logging middleware
use axum::{
    extract::Request,
    http::{status::StatusCode, HeaderValue},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::slice::Iter;

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

/// TODO: Github status handler
///
/// TODO: Discord status handler
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

fn handle_signature_256(val: &HeaderValue) -> Result<(), ()> {
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
pub async fn github_webhook_handler(request: Request) -> impl IntoResponse {
    let headers = request.headers();

    if headers.is_empty() {
        return StatusCode::BAD_REQUEST;
    }

    for h in GithubHeaders::iterator() {
        match headers.get(h.as_str()) {
            Some(value) => match h {
                GithubHeaders::UserAgent => {
                    let Ok(_) = handle_user_agent_value(value) else {
                        return StatusCode::BAD_REQUEST;
                    };
                }
                GithubHeaders::Signature256 => {
                    let Ok(_) = handle_signature_256(value) else {
                        return StatusCode::BAD_REQUEST;
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
            None => return StatusCode::BAD_REQUEST,
        }
    }

    StatusCode::ACCEPTED
}

pub fn create_app() -> Router {
    Router::new()
        .route("/", get(root))
        .route("/gh", post(github_webhook_handler))
}

pub async fn create_listener(port: u16) -> tokio::net::TcpListener {
    let addr = format!("0.0.0.0:{}", port);
    tokio::net::TcpListener::bind(addr).await.unwrap()
}
