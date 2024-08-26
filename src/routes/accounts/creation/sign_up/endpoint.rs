use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use scylla::Session;
use serde::Deserialize;
use tokio::task;

use crate::common::{birth_year::BirthYear, email::Email, language::Language, password::Password, region::Region};

use super::r#impl::SignUpImpl;

use crate::routes::accounts::creation::sign_up::dsl::SignUp;

pub async fn sign_up_route(db: Arc<Session>) -> Router {
    // ここは`unwrap()`にすべきではない？
    let exists_by_email = db.prepare("SELECT id FROM accounts_by_email WHERE email = ?").await.unwrap();
    let insert_creation_application = db.prepare("INSERT INTO account_creation_applications (email, password_hash, region, language, birth_year, code) VALUES (?, ?, ?, ?, ?, ?) USING TTL 86400").await.unwrap();

    let routine = SignUpImpl::new(
        db,
        Arc::new(exists_by_email),
        Arc::new(insert_creation_application)
    );

    Router::new()
        .route("/sign_up", post(handler))
        .with_state(Arc::new(routine))
}

pub async fn handler(
    State(routine): State<Arc<SignUpImpl>>,
    Json(payload): Json<Payload>,
) -> impl IntoResponse {
    // quick exit 対策
    task::spawn(async move {
        // 結果をログに記録
        routine.sign_up(&payload.email, &payload.password, &payload.birth_year, &payload.region, &payload.language).await;
    });
    StatusCode::OK
}

#[derive(Deserialize)]
pub struct Payload {
    pub email: Email,
    pub password: Password,
    pub region: Region,
    pub language: Language,
    pub birth_year: BirthYear,
}