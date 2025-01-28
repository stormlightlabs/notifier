use std::env;

use server::{create_app, create_listener};

mod bot;
mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let _ = env::var("CLOUDFLARE_TUNNEL_TOKEN")
        .expect("Expected a token for the cloudflare ingress in the environment");

    let _ = env::var("DISCORD_TOKEN")
        .expect("Expected a token for the cloudflare ingress in the environment");

    let app = create_app();
    let listener = create_listener(4040).await;

    axum::serve(listener, app).await.unwrap();
}

#[cfg(test)]
mod tests {
    use crate::server;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
    };
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_root_endpoint() {
        let app = server::create_app();
        let response = app
            .oneshot(Request::builder().uri("/").body(Body::empty()).unwrap())
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_not_found() {
        let app = server::create_app();
        let response = app
            .oneshot(
                Request::builder()
                    .uri("/non-existent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
