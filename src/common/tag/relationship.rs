#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TagRelationType {
    Super,
    Equivalent,
    Sub,
}