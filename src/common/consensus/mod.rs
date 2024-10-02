use proposal::IsProposal;
use stability::Stability;

pub mod proposal;
pub mod stability;

pub fn is_unstable_proposal(is_proposal: IsProposal, is_stable: Stability) -> bool {
    is_proposal == IsProposal::Proposal && is_stable == Stability::Unstable
}