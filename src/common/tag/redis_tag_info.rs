pub struct RedisTagInfo(u32);

impl RedisTagInfo {
    pub fn construct(order: TagListOrder, ratings_sum: u32, is_proposal: bool, is_stable: bool) -> Self {
        let order_bits = (order as u32 & 0b11) << 30;          // 上位2ビット
        let ratings_sum_bits = (ratings_sum & 0x0FFFFFFF) << 2; // 次の28ビット
        let is_proposal_bit = (is_proposal as u32 & 0b1) << 1;  // ビット1
        let is_stable_bit = is_stable as u32 & 0b1;          // ビット0

        // 全てのビットを結合
        let value = order_bits | ratings_sum_bits | is_proposal_bit | is_stable_bit;

        Self(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }

    /// `order` フィールドを取得します。
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

    /// `ratings_sum` フィールドを取得します。
    pub fn ratings_sum(&self) -> u32 {
        // ビット29-2を取得
        (self.0 >> 2) & 0x0FFFFFFF
    }

    /// `is_proposal` フィールドを取得します。
    pub fn is_proposal(&self) -> bool {
        // ビット1を取得
        ((self.0 >> 1) & 0b1) != 0
    }

    /// `is_stable` フィールドを取得します。
    pub fn is_stable(&self) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_construct_and_getters() {
        let order = TagListOrder::NormalUnstable;
        let ratings_sum = 0x0ABCDE; // 28ビット以内
        let is_proposal = true;
        let is_stable = false;

        let tag_info = RedisTagInfo::construct(order, ratings_sum, is_proposal, is_stable);

        // 構造体の値を直接確認
        let expected_value = ((order as u32 & 0b11) << 30)
            | ((ratings_sum & 0x0FFFFFFF) << 2)
            | ((is_proposal as u32 & 0b1) << 1)
            | (is_stable as u32 & 0b1);
        assert_eq!(tag_info.value(), expected_value);

        // ゲッターメソッドをテスト
        assert_eq!(tag_info.order(), order);
        assert_eq!(tag_info.ratings_sum(), ratings_sum);
        assert_eq!(tag_info.is_proposal(), is_proposal);
        assert_eq!(tag_info.is_stable(), is_stable);
    }

    #[test]
    fn test_max_values() {
        let order = TagListOrder::ReachableTagOrValidProposalOrUncalcProposal;
        let ratings_sum = 0x0FFFFFFF; // 最大28ビット
        let is_proposal = true;
        let is_stable = true;

        let tag_info = RedisTagInfo::construct(order, ratings_sum, is_proposal, is_stable);

        assert_eq!(tag_info.order(), order);
        assert_eq!(tag_info.ratings_sum(), ratings_sum);
        assert_eq!(tag_info.is_proposal(), is_proposal);
        assert_eq!(tag_info.is_stable(), is_stable);
    }

    #[test]
    fn test_min_values() {
        let order = TagListOrder::InvalidUnstable;
        let ratings_sum = 0;
        let is_proposal = false;
        let is_stable = false;

        let tag_info = RedisTagInfo::construct(order, ratings_sum, is_proposal, is_stable);

        assert_eq!(tag_info.order(), order);
        assert_eq!(tag_info.ratings_sum(), ratings_sum);
        assert_eq!(tag_info.is_proposal(), is_proposal);
        assert_eq!(tag_info.is_stable(), is_stable);
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