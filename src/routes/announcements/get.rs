/*use std::collections::HashSet;

use axum::{extract::{Query, State}, response::IntoResponse, routing::get, Json, Router};
use redis::{aio::MultiplexedConnection, AsyncCommands, Client};
use scylla::{prepared_statement::PreparedStatement, Session};
use serde_json::{from_str, Value};
use uuid::Uuid;

use crate::sugar::domain::{language::Language, region::Region};

use super::domain::Announcement;

/*pub fn route(scylla: Session, garnet: Client) -> Router<()> {
  Router::new()
    .route("/announcements", get(handler))
    .with_state(MockAnnouncementsRepository)
}*/


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

impl AnnouncementsRepository for MockAnnouncementsRepository {
  fn get(&self, language: Language, region: Region) -> AnnouncementsJson {
    let announcement = Announcement {
      id: Uuid::now_v7(),
      title: "テストお知らせ".to_owned()
    };

    let s = serde_json::to_string(&announcement);

    match s {
      Ok(json) => json,
      Err(_) => "".to_owned()
    }
  }
}

struct RemoteAnnouncementsRepository {
  pub db: Session,
  // pub cache: MultiplexedConnection,
}

//const ANNOUNCEMENTS_CACHE_NAMESPACE: &str = "announcements";

impl AnnouncementsRepository for RemoteAnnouncementsRepository {
  async fn get(&mut self, language: Language, region: Region) -> AnnouncementsJson {
    let prepared: PreparedStatement = self
      .db
      .prepare("SELECT id, title FROM recent_announcements WHERE language = ? AND region = ? AND id >= ?")
      .await
      .unwrap(); // unwrap()ではいけない、同上

    "".to_owned()
  }
}

/*fn should_use_alternative_region(language: &Language, region: &Region) -> bool {
  match (language, region) {
    (Language::Japanese, Region::Japan) => false,
    (Language::Korean, Region::RepublicOfKorea) => false,
    (Language::TaiwaneseMandarin, Region::Taiwan) => false,
    (Language::AmericanEnglish, Region::UnitedStatesOfAmerica) => false,
    _ => true,
  }
}*/

#[cfg(test)]
mod tests {

}
*/