use std::sync::Arc;

use axum::{extract::State, routing::post, Form, Json, Router};
use http::StatusCode;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::error;

use crate::{common::{api_key::key::ApiKey, turnstile::TurnstileToken}, helper::{error::InitError, redis::connection::Pool}};

use super::{dsl::{IssueApiKey, IssueApiKeyError}, interpreter::IssueApiKeyImpl};

// ここにレート制限がかけられないので、WAFなどで設定する必要がある
pub async fn endpoint(cache: Arc<Pool>, client: Arc<Client>, turnstile_secret_key: String) -> Result<Router, InitError<IssueApiKeyImpl>> {
    let sign_in = IssueApiKeyImpl::try_new(cache, client, turnstile_secret_key).await?;

    let router = Router::new()
        .route("/", post(handler))
        .with_state(Arc::new(sign_in));

    Ok(router)
}

pub async fn handler(
    State(routine): State<Arc<IssueApiKeyImpl>>,
    Form(form): Form<TurnstileForm>,
) -> Result<Json<Data>, StatusCode> {
    match routine.issue_api_key(&TurnstileToken::new(form.cf_turnstile_token)).await {
        Ok(api_key) => Ok(Json(Data { api_key })),
        Err(e) => match e {
            IssueApiKeyError::InvalidToken => Err(StatusCode::BAD_REQUEST),
            e => {
                error!(
                    error = %e,
                    "APIキーの発行に失敗しました"
                );

                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        }
    }
}

#[derive(Deserialize)]
pub struct TurnstileForm {
    #[serde(rename = "cf-turnstile-token")]
    cf_turnstile_token: String,
}

#[derive(Serialize)]
pub struct Data {
    api_key: ApiKey,
}