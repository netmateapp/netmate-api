use std::{str::FromStr, sync::Arc};

use rand::{RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;
use scylla::{prepared_statement::PreparedStatement, Session};

use crate::common::{birth_year::BirthYear, email::Email, language::Language, password::{Password, PasswordHash}, region::Region, send_email::{Body, NetmateEmail, ResendEmailService, SenderNameLocale, Subject, TransactionalEmailService}};

// axumのrouterを返す関数
// quick exit 対策はここで行い、アプリケーションには波及させない
// 返された成否情報をもとにロギング

/*pub fn route(db: Arc<Session>) -> Router<()> {
    Router::new()
        .route("/sign_up", post(handler))
        .with_state(state)
}*/

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

struct OneTimeToken(String);

impl OneTimeToken {
    pub fn value(&self) -> &String {
        &self.0
    }

    pub fn generate() -> OneTimeToken {
        let charset: Vec<char> = "abcdefghijklmnopqrstuvwxyz012345".chars().collect();

        let mut rng = ChaCha20Rng::from_entropy();
        let mut random_bytes = [0u8; 15];
        rng.fill_bytes(&mut random_bytes);

        let mut token = String::with_capacity(24);
        let mut bit_buffer: u32 = 0;
        let mut bit_count = 0;

        for byte in random_bytes.iter() {
            bit_buffer |= (*byte as u32) << bit_count;
            bit_count += 8;

            while bit_count >= 5 {
                let index = (bit_buffer & 0x1F) as usize;
                token.push(charset[index]);
                bit_buffer >>= 5;
                bit_count -= 5;
            }
        }

        if bit_count > 0 {
            token.push(charset[(bit_buffer & 0x1F) as usize]);
        }

        OneTimeToken(token)
    }
}

#[derive(Debug, thiserror::Error)]
enum SignUpError {
    #[error("指定のメールアドレスが利用可能である保証が得られませんでした")]
    PotentiallyUnavailableEmail(#[source] anyhow::Error),
    #[error("指定のメールアドレスは利用不能です")]
    UnavaialbleEmail,
    #[error("アカウント作成の申請に失敗しました")]
    ApplicationFailed(#[source] anyhow::Error),
    #[error("認証メールの送信に失敗しました")]
    AuthenticationEmailSendFailed(#[source] anyhow::Error)
}

// ネットワーク環境を前提とした設計
// 純粋な論理ではなく、現実的な構造をモデル化する必要がある
// 全ての関数呼び出し=ネットワーク越し処理は失敗する可能性がある (=ローカル性が無い)
// ローカル性のある環境では？ → 失敗可能性を否定できないものにResultがつく
// => 確実に成功する保証がなければResult
// 失敗可能性があるのなら、実際の失敗の種類も型にできる(詳細はsourceで保持)

type Fallible<T, E> = Result<T, E>;

trait SignUp {
    async fn sign_up(&self, email: &Email, password: &Password, birth_year: &BirthYear, region: &Region, language: &Language) -> Fallible<(), SignUpError> {
        if self.is_available_email(email).await? {
            // この位置でパスワードのハッシュ化を行う必要があり高い負荷が発生するため、
            // `sign_up`は自動化されたリクエストから特に保護されなければならない
            let hash: PasswordHash = password.hashed();
            let token = OneTimeToken::generate();
            self.apply_to_create_account(email, &hash, birth_year, region, language, &token).await?;
            self.send_verification_email(email, language, &token).await
        } else {
            Err(SignUpError::UnavaialbleEmail)
        }
    }

    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError>;

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language, token: &OneTimeToken) -> Fallible<(), SignUpError>;

    async fn send_verification_email(&self, email: &Email, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError>;
}

struct SignUpImpl {
    session: Arc<Session>,
    exists_by_email: Arc<PreparedStatement>,
    insert_creation_application: Arc<PreparedStatement>,
}

impl SignUp for SignUpImpl {
    async fn is_available_email(&self, email: &Email) -> Fallible<bool, SignUpError> {
        let res = self.session
            // ここに clone() が必要で、clone()を強制する設計が必要
            .execute(&self.exists_by_email, (email.value(), ))
            .await;

        match res {
            Ok(qr) => match qr.rows() {
                Ok(v) => Ok(!v.is_empty()),
                Err(e) => Err(SignUpError::PotentiallyUnavailableEmail(e.into()))
            },
            Err(e) => Err(SignUpError::PotentiallyUnavailableEmail(e.into()))
        }
    }

    async fn apply_to_create_account(&self, email: &Email, pw_hash: &PasswordHash, birth_year: &BirthYear, region: &Region, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        let encoded_birth_year: i16 = (*birth_year).into();
        let encoded_region: i8 = (*region).into();
        let encoded_language: i8 = (*language).into();

        let res = self.session
            .execute(&self.insert_creation_application, (token.value(), email.value(), pw_hash.value(), encoded_birth_year, encoded_region, encoded_language,))
            .await;

        res.map(|_| ())
            .map_err(|e| SignUpError::ApplicationFailed(e.into()))
    }

    async fn send_verification_email(&self, email: &Email, language: &Language, token: &OneTimeToken) -> Result<(), SignUpError> {
        let sender_name = &SenderNameLocale::expressed_in(language);

        let from = match Email::from_str("verify-email@account.netmate.app") {
            Ok(email) => match NetmateEmail::try_from(email) {
                Ok(ne) => ne,
                Err(e) => return Err(SignUpError::AuthenticationEmailSendFailed(e.into())),
            },
            Err(e) => return Err(SignUpError::AuthenticationEmailSendFailed(e.into()))
        };

        // バックエンドの多言語対応もロケールファイルを作成し、そこから取得すべきでは
        let subject = match language {
            Language::Japanese => "",
            _ => ""
        };
        let subject = match Subject::from_str(&subject) {
            Ok(s) => s,
            Err(e) => return Err(SignUpError::AuthenticationEmailSendFailed(e.into()))
        };

        let body = Body::new("", "");
        
        ResendEmailService::send(sender_name, &from, &email, &subject, &body)
            .await
            .map_err(|e| SignUpError::AuthenticationEmailSendFailed(e.into()))
    }
}

// mockを使用した自動テスト

#[cfg(test)]
mod tests {
}