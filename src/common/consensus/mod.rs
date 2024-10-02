use proposal::ItemType;
use stability::Stability;

pub mod proposal;
pub mod stability;

pub fn is_unstable_proposal(is_proposal: ItemType, is_stable: Stability) -> bool {
    is_proposal == ItemType::Proposal && is_stable == Stability::Unstable
}