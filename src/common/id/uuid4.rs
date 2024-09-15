use serde::Serialize;
use thiserror::Error;
use uuid::Uuid;

pub struct Uuid4(Uuid);

impl Uuid4 {
    pub const fn new_unchecked(uuidv4: Uuid) -> Uuid4 {
        Uuid4(uuidv4)
    }

    pub fn value(&self) -> &Uuid {
        &self.0
    }
}

#[derive(Debug, Error)]
#[error("UUIDのバージョンが4ではありません")]
pub struct ParseUuid4Error;

impl TryFrom<Uuid> for Uuid4 {
    type Error = ParseUuid4Error;

    fn try_from(value: Uuid) -> Result<Self, Self::Error> {
        if value.get_version_num() == 7 {
            Ok(Uuid4(value))
        } else {
            Err(ParseUuid4Error)
        }
    }
}

impl Serialize for Uuid4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer
    {
        self.0.serialize(serializer)
    }
}