use std::sync::Arc;

use axum::{routing::post, Router};
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::common::{birth_year::BirthYear, email::Email, language::Language, region::Region};

// axumのrouterを返す関数
// quick exit 対策はここで行い、アプリケーションには波及させない
// 返された成否情報をもとにロギング

pub fn route(db: Arc<Session>) -> Router<()> {
    Router::new()
        .route("/sign_up", post(handler))
        .with_state(state)
}

/*
pub fn route(scylla: Session, garnet: Client) -> Router<()> {
  Router::new()
    .route("/announcements", get(handler))
    .with_state(MockAnnouncementsRepository)
}

pub async fn handler(
    query: Query<Params>,
    State(infra): State<impl AnnouncementsRepository>
  ) -> impl IntoResponse {
    let str = infra.get(query.0.lang, query.0.reg);
    let json: Value = from_str(&str).unwrap();
    Json(json)
  }
  
  struct Params {
    pub lang: Language,
    pub reg: Region,
  }
  
  type AnnouncementsJson = String;
  
  trait AnnouncementsRepository {
    async fn get(&mut self, language: Language, region: Region) -> AnnouncementsJson;
  }
  
  #[derive(Clone)]
  struct MockAnnouncementsRepository;
  
*/

struct Password(String);

#[derive(Debug, thiserror::Error)]
enum SignUpError {
    #[error("指定のメールアドレスが利用可能である保証が得られませんでした")]
    PotentiallyUnavailableEmail(#[source] anyhow::Error),
    #[error("指定のメールアドレスは利用不能です")]
    UnavaialbleEmail,
    #[error("アカウント作成の申請に失敗しました")]
    FailedApplication(#[source] std::io::Error),
}


// ネットワーク環境を前提とした設計
// 純粋な論理ではなく、現実的な構造をモデル化する必要がある
// 全ての関数呼び出し=ネットワーク越し処理は失敗する可能性がある (=ローカル性が無い)
// ローカル性のある環境では？ → 失敗可能性を否定できないものにResultがつく
// => 確実に成功する保証がなければResult

type Fallible<T, E> = Result<T, E>;

trait SignUp {
    async fn sign_up(&self, email: &Email, password: &Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), SignUpError> {
        if self.is_available_email(email).await? {
            self.apply_to_create_account(email, password, birth_year, region, language).await
        } else {
            Err(SignUpError::UnavaialbleEmail)
        }
    }

    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError>;

    async fn apply_to_create_account(&self, email: &Email, password: &Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), SignUpError>;
}

struct SignUpImpl {
    session: Arc<Session>,
    exists_by_email: Arc<PreparedStatement>,
    insert_application: Arc<PreparedStatement>,
}

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Result<(), SignUpError> {
        let res = self.session
            .execute(&self.exists_by_email, (email.value(), ))
            .await;

        match res {
            Ok(qr) => match qr.rows() {
                Ok(v) => if v.is_empty() { Ok(()) } else { Err(SignUpError::PotentiallyUnavailableEmail(std::error::Error::)) }
                Err(e) => Err(SignUpError::PotentiallyUnavailableEmail(e.into()))
            },
            Err(e) => Err(SignUpError::PotentiallyUnavailableEmail(e.into()))
        }
    }

    fn apply_to_create_account(&self, email: &Email, password: &Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Result<(), SignUpError> {
        
    }
}

// mockを使用した自動テスト

#[cfg(test)]
mod tests {
    use crate::common::{birth_year::BirthYear, email::Email, language::Language, region::Region};

    use super::SignUp;

    struct MockSignUp;

    /*impl SignUp for MockSignUp {
        fn is_available_email(email: &Email) -> bool {
            
        }

        fn apply_to_create_account(email: &Email, password: &super::Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Result<(), super::SignUpError> {
            
        }
    }*/
}