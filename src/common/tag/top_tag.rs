use uuid::Uuid;

use crate::common::{language::Language, uuid::uuid4::Uuid4};

use super::tag_id::TagId;

const JAPANESE_TOP_TAG: TagId = top_tag_id(0x00);
const KOREAN_TOP_TAG: TagId = top_tag_id(0x01);
const TAIWANESE_MANDARIN_TOP_TAG: TagId = top_tag_id(0x02);
const AMERICAN_ENGLISH_TOP_TAG: TagId = top_tag_id(0x03);

const fn top_tag_id(d4_8: u8) -> TagId {
    TagId::new(Uuid4::new_unchecked(Uuid::from_fields(0x00, 0x00, 0x4000, &[0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, d4_8])))
}

pub const fn top_tag_id_by_language(language: Language) -> TagId {
    match language {
        Language::Japanese => JAPANESE_TOP_TAG,
        Language::Korean => KOREAN_TOP_TAG,
        Language::TaiwaneseMandarin => TAIWANESE_MANDARIN_TOP_TAG,
        _ => AMERICAN_ENGLISH_TOP_TAG,
    }
}

pub fn is_top_tag(tag_id: TagId) -> bool {
    let uuid = tag_id.value().value();
    let bytes = uuid.as_bytes();
    bytes[0..5] == [0, 0, 0, 0, 0, 0] &&
    bytes[6] == 0x40 &&
    bytes[7] == 0 &&
    bytes[8] == 0x80 &&
    bytes[9..14] == [0, 0, 0, 0, 0, 0] &&
    bytes[15] < 4
}

#[cfg(test)]
mod tests {
    use uuid::Variant;

    use crate::common::tag::{tag_id::TagId, top_tag::{is_top_tag, top_tag_id, AMERICAN_ENGLISH_TOP_TAG, JAPANESE_TOP_TAG, KOREAN_TOP_TAG, TAIWANESE_MANDARIN_TOP_TAG}};

    #[test]
    fn check_top_tag_id_format() {
        let top_tag_id = top_tag_id(0x00);
        assert_eq!(top_tag_id.value().value().get_version_num(), 4);
        assert_eq!(top_tag_id.value().value().get_variant(), Variant::RFC4122);
    }

    #[test]
    fn test_is_top_tag() {
        for top_tag_id in [JAPANESE_TOP_TAG, KOREAN_TOP_TAG, TAIWANESE_MANDARIN_TOP_TAG, AMERICAN_ENGLISH_TOP_TAG] {
            assert!(is_top_tag(top_tag_id));
        }
    }

    #[test]
    fn check_top_tag_ids() {
        fn check_top_tag_id(top_tag_id: TagId, d4_8: u8) {
            assert_eq!(top_tag_id.value().value().as_fields().3[7], d4_8);
        }

        check_top_tag_id(JAPANESE_TOP_TAG, 0x00);
        check_top_tag_id(KOREAN_TOP_TAG, 0x01);
        check_top_tag_id(TAIWANESE_MANDARIN_TOP_TAG, 0x02);
        check_top_tag_id(AMERICAN_ENGLISH_TOP_TAG, 0x03);
    }
}