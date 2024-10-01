use redis::{FromRedisValue, RedisResult, RedisWrite, ToRedisArgs};

pub struct RedisTagInfo(u32);

impl RedisTagInfo {
    pub fn construct(order: TagListOrder, ratings_sum: u32, is_proposal: bool, is_stable: bool, is_status_calculated: bool) -> Self {
        let order_bits = (order as u32 & 0b11) << 30;          // 上位2ビット
        let ratings_sum_bits = (ratings_sum & 0x07FFFFFF) << 3; // 次の27ビット
        let is_proposal_bit = (is_proposal as u32 & 0b1) << 2;  // ビット2
        let is_stable_bit = (is_stable as u32 & 0b1) << 1;          // ビット1
        let is_status_calculated_bit = is_status_calculated as u32 & 0b1; // ビット0

        // 全てのビットを結合
        let value = order_bits | ratings_sum_bits | is_proposal_bit | is_stable_bit | is_status_calculated_bit;

        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn order(&self) -> TagListOrder {
        // 上位2ビットを取得
        let order_bits = (self.0 >> 30) & 0b11;
        match order_bits {
            2 => TagListOrder::ReachableTagOrValidProposalOrUncalcProposal,
            1 => TagListOrder::NormalUnstable,
            0 => TagListOrder::InvalidUnstable,
            _ => unreachable!("Invalid order bits"),
        }
    }

    pub fn ratings_sum(&self) -> u32 {
        // ビット29-3を取得
        (self.0 >> 3) & 0x07FFFFFF
    }

    pub fn is_proposal(&self) -> bool {
        // ビット2を取得
        ((self.0 >> 2) & 0b1) != 0
    }

    pub fn is_stable(&self) -> bool {
        // ビット1を取得
        ((self.0 >> 1) & 0b1) != 0
    }

    pub fn is_status_calculated(&self) -> bool {
        // ビット0を取得
        (self.0 & 0b1) != 0
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagListOrder {
    ReachableTagOrValidProposalOrUncalcProposal = 2,
    NormalUnstable = 1,
    InvalidUnstable = 0,
}

impl ToRedisArgs for RedisTagInfo {
    fn write_redis_args<W: ?Sized + RedisWrite>(&self, out: &mut W) {
        u32::write_redis_args(&self.value(), out);
    }
}

impl FromRedisValue for RedisTagInfo {
    fn from_redis_value(v: &redis::Value) -> RedisResult<Self> {
        u32::from_redis_value(v).map(RedisTagInfo)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_and_getters() {
        let order = TagListOrder::NormalUnstable;
        let ratings_sum = 0x0ABCDE; // 27ビット以内
        let is_proposal = true;
        let is_stable = false;
        let is_status_calculated = true;

        let tag_info = RedisTagInfo::construct(order, ratings_sum, is_proposal, is_stable, is_status_calculated);

        // 構造体の値を直接確認
        let expected_value = ((order as u32 & 0b11) << 30)
            | ((ratings_sum & 0x07FFFFFF) << 3)
            | ((is_proposal as u32 & 0b1) << 2)
            | ((is_stable as u32 & 0b1) << 1)
            | (is_status_calculated as u32 & 0b1);
        assert_eq!(tag_info.value(), expected_value);

        // ゲッターメソッドをテスト
        assert_eq!(tag_info.order(), order);
        assert_eq!(tag_info.ratings_sum(), ratings_sum);
        assert_eq!(tag_info.is_proposal(), is_proposal);
        assert_eq!(tag_info.is_stable(), is_stable);
        assert_eq!(tag_info.is_status_calculated(), is_status_calculated);
    }

    #[test]
    fn test_max_values() {
        let order = TagListOrder::ReachableTagOrValidProposalOrUncalcProposal;
        let ratings_sum = 0x07FFFFFF; // 最大27ビット
        let is_proposal = true;
        let is_stable = true;
        let is_status_calculated = true;

        let tag_info = RedisTagInfo::construct(order, ratings_sum, is_proposal, is_stable, is_status_calculated);

        assert_eq!(tag_info.order(), order);
        assert_eq!(tag_info.ratings_sum(), ratings_sum);
        assert_eq!(tag_info.is_proposal(), is_proposal);
        assert_eq!(tag_info.is_stable(), is_stable);
        assert_eq!(tag_info.is_status_calculated(), is_status_calculated);
    }

    #[test]
    fn test_min_values() {
        let order = TagListOrder::InvalidUnstable;
        let ratings_sum = 0;
        let is_proposal = false;
        let is_stable = false;
        let is_status_calculated = false;

        let tag_info = RedisTagInfo::construct(order, ratings_sum, is_proposal, is_stable, is_status_calculated);

        assert_eq!(tag_info.order(), order);
        assert_eq!(tag_info.ratings_sum(), ratings_sum);
        assert_eq!(tag_info.is_proposal(), is_proposal);
        assert_eq!(tag_info.is_stable(), is_stable);
        assert_eq!(tag_info.is_status_calculated(), is_status_calculated);
    }

    #[test]
    #[should_panic(expected = "Invalid order bits")]
    fn test_invalid_order_bits() {
        // 例えば、order_bitsが3の場合は未定義
        let invalid_value = 3u32 << 30;
        let tag_info = RedisTagInfo(invalid_value);
        let _ = tag_info.order(); // panicするはず
    }
}