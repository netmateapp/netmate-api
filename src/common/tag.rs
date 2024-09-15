use uuid::Uuid;

use super::{id::tag_id::TagId, language::Language, uuid::uuid4::Uuid4};

const JAPANESE_TOP_TAG: TagId = TagId::new(Uuid4::new_unchecked(Uuid::from_fields(0x00, 0x00, 0x4000, &[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])));

pub const fn top_tag_id_by_language(_language: &Language) -> TagId {
    JAPANESE_TOP_TAG
}

#[cfg(test)]
mod tests {
    use uuid::Variant;

    use crate::common::tag::JAPANESE_TOP_TAG;

    #[test]
    fn check_top_tag_id_format() {
        assert_eq!(JAPANESE_TOP_TAG.value().value().get_version_num(), 4);
        assert_eq!(JAPANESE_TOP_TAG.value().value().get_variant(), Variant::RFC4122);
    }
}