use serde::{Serialize, Serializer};
use thiserror::Error;
use uuid::Uuid;

use crate::common::{language_group::LanguageGroup, uuid::uuid4::Uuid4};

use super::tag_id::TagId;

const JAPANESE: TopTagId = of(LanguageGroup::Japanese);
const KOREAN: TopTagId = of(LanguageGroup::Korean);
const TAIWANESE_MANDARIN: TopTagId = of(LanguageGroup::TaiwaneseMandarin);
const ENGLISH: TopTagId = of(LanguageGroup::English);

pub struct TopTagId(TagId);

const fn of(group: LanguageGroup) -> TopTagId {
    let uuid = Uuid::from_fields(0x00, 0x00, 0x4000, &[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, group.as_u8()]);
    let uuidv4 = Uuid4::new_unchecked(uuid);
    let tag_id = TagId::of(uuidv4);
    TopTagId(tag_id)
}

impl TopTagId {
    pub fn value(&self) -> TagId {
        self.0
    }
}

impl From<LanguageGroup> for TopTagId {
    fn from(value: LanguageGroup) -> Self {
        match value {
            LanguageGroup::Japanese => JAPANESE,
            LanguageGroup::Korean => KOREAN,
            LanguageGroup::TaiwaneseMandarin => TAIWANESE_MANDARIN,
            LanguageGroup::English => ENGLISH,
        }
    }
}

pub fn is_top_tag_id(tag_id: TagId) -> bool {
    let uuid = tag_id.value().value();
    let bytes = uuid.as_bytes();
    bytes[0..=5] == [0, 0, 0, 0, 0, 0] && bytes[6] == 0x40 && bytes[7] == 0 && bytes[8] == 0x80 && bytes[9..=14] == [0, 0, 0, 0, 0, 0] && bytes[15] < 4
}

#[derive(Debug, Error, PartialEq)]
#[error("トップタグIDの解析に失敗しました")]
pub struct ParseTopTagIdError;

impl TryFrom<TagId> for TopTagId {
    type Error = ParseTopTagIdError;

    fn try_from(value: TagId) -> Result<Self, Self::Error> {
        let uuid = value.value().value();
        let bytes = uuid.as_bytes();
        
        if bytes[0..=5] == [0, 0, 0, 0, 0, 0] && bytes[6] == 0x40 && bytes[7] == 0 && bytes[8] == 0x80 && bytes[9..=14] == [0, 0, 0, 0, 0, 0] && bytes[15] < 4 {
            Ok(Self(value))
        } else {
            Err(ParseTopTagIdError)
        }
    }
}

impl Serialize for TopTagId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        Serialize::serialize(&self.value(), serializer)
    }
}

#[cfg(test)]
mod tests {
    use uuid::Variant;

    use crate::common::{language_group::LanguageGroup, tag::top_tag::{of, TopTagId, ENGLISH, JAPANESE, KOREAN, TAIWANESE_MANDARIN}};

    #[test]
    fn check_top_tag_id_format() {
        let top_tag_id = of(LanguageGroup::Japanese);
        assert_eq!(top_tag_id.value().value().value().get_version_num(), 4);
        assert_eq!(top_tag_id.value().value().value().get_variant(), Variant::RFC4122);
    }

    #[test]
    fn test_is_top_tag() {
        for top_tag_id in [JAPANESE, KOREAN, TAIWANESE_MANDARIN, ENGLISH] {
            assert!(TopTagId::try_from(top_tag_id.value()).is_ok());
        }
    }

    #[test]
    fn check_top_tag_ids() {
        fn check_top_tag_id(top_tag_id: TopTagId, d4_8: u8) {
            assert_eq!(top_tag_id.value().value().value().as_fields().3[7], d4_8);
        }

        check_top_tag_id(JAPANESE, 0x00);
        check_top_tag_id(KOREAN, 0x01);
        check_top_tag_id(TAIWANESE_MANDARIN, 0x02);
        check_top_tag_id(ENGLISH, 0x03);
    }
}