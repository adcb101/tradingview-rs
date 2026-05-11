pub mod batch;
pub mod single;

/// Resolve authentication token from parameter or environment.
/// Falls back to "unauthorized_user_token" when JWT is unavailable —
/// session/signature cookie auth is handled separately in setup_websocket.
fn resolve_auth_token(auth_token: Option<&str>) -> String {
    auth_token
        .map(|t| t.to_string())
        .or_else(|| std::env::var("TV_AUTH_TOKEN").ok())
        .unwrap_or_else(|| {
            tracing::debug!("No TV_AUTH_TOKEN set, will rely on session/signature cookie auth");
            "unauthorized_user_token".to_string()
        })
}
