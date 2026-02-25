use crate::{
    db::operations::is_token_valid, error::AuthError, state::AppState, utils::get_token::get_token,
};
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: String,
}

pub async fn auth(
    State(state): State<AppState>,
    mut req: Request,
    next: Next,
) -> Result<Response, AuthError> {
    let token = get_token(req.headers()).ok_or(AuthError::NoTokenProvided)?;

    let user_id = is_token_valid(&state.pool, &token)
        .await
        .map_err(|e| AuthError::DatabaseError(e))?
        .ok_or(AuthError::InvalidToken)?;

    tracing::debug!("Successful auth for token: {}...", &token[..8]);

    req.extensions_mut().insert(AuthUser { user_id });

    Ok(next.run(req).await)
}
