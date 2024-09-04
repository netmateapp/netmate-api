use serde::Serialize;
use uuid::Uuid;

pub type Id = Uuid;

#[derive(Serialize)]
pub struct Announcement {
  pub id: Id,
  pub title: String,
}
