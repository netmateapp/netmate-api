use crate::common::{id::account_id::AccountId, tag::{relation::TagRelation, tag_id::TagId}};

pub(crate) trait GetTagRelationRating {
    async fn get_tag_relation_rating(&self, account_id: AccountId, subtag_id: TagId, supertag_id: TagId, relation: TagRelation) {
        
    }
}