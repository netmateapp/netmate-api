use uuid::Uuid;

use crate::common::{tag::{non_top_tag::NonTopTagId, tag_id::TagId}, uuid::uuid4::Uuid4};

#[cfg(debug_assertions)]
pub const fn mock_uuid(d4_8: u8) -> Uuid {
    Uuid::from_fields(0x01, 0x01, 0x4001, &[0x80, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, d4_8])
}

#[cfg(debug_assertions)]
pub fn mock_non_top_tag_id(d4_8: u8) -> NonTopTagId {
    NonTopTagId::try_from(TagId::of(Uuid4::new_unchecked(mock_uuid(d4_8)))).unwrap()
}