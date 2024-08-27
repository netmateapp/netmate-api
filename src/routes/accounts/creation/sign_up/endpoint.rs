use std::sync::Arc;

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::post, Json, Router};
use scylla::Session;
use serde::Deserialize;
use tokio::task;
use tracing::info;

use crate::common::{birth_year::BirthYear, email::Email, language::Language, password::Password, region::Region};

use super::dsl::SignUp;
use super::interpreter::SignUpImpl;

pub async fn endpoint(db: Arc<Session>) -> Router {
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
    // 非 quick exit パターンを採用し、攻撃者に処理時間の差を計測させない
    task::spawn(async move {
        match routine.sign_up(&payload.email, &payload.password, &payload.birth_year, &payload.region, &payload.language).await {
            Ok(_) => info!(
                email = %payload.email.value(),
                "アカウント作成の申請が正常に処理されました。"
            ),
            Err(e) => info!(
                email = %payload.email.value(),
                // 生年は重要な個人情報であるため、メールアドレスと紐付けて記録してはならない
                region = %u8::from(payload.region),
                language = %u8::from(payload.language),
                error = %e,
                "アカウント作成の申請に失敗しました。"
            ),
        }
    });

    // `sign_up`の終了を待たずに返す
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