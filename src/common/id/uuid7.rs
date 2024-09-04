use thiserror::Error;
use uuid::Uuid;

#[derive(Clone)]
pub struct Uuid7(Uuid);

impl Uuid7 {
    pub fn now() -> Uuid7 {
        Uuid7(Uuid::now_v7())
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("UUIDのバージョンが7ではありません")]
pub struct ParseUuid7Error;

impl TryFrom<Uuid> for Uuid7 {
    type Error = ParseUuid7Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        if value.get_version_num() == 7 {
            Ok(Uuid7(value))
        } else {
            Err(ParseUuid7Error)
        }
    }
}