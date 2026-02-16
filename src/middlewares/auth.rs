use crate::{db::operations::is_token_valid, error::AuthError, state::AppState};
use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};

pub async fn auth(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = req
        .headers()
        .get(AUTHORIZATION)
        .and_then(|h| h.to_str().ok())
        .and_then(|h| h.strip_prefix("Bearer "))
        .ok_or(AuthError::NoTokenProvided)?;

    is_token_valid(&state.pool, token)
        .await
        .map_err(|e| AuthError::DatabaseError(e))?
        .then_some(())
        .ok_or(AuthError::InvalidToken)?;

    tracing::debug!("Successful auth for token: {}...", &token[..8]);

    Ok(next.run(req).await)
}
