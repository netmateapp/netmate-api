#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct HandleShareCount(u32);

impl HandleShareCount {
    pub const fn of(value: u32) -> Self {
        HandleShareCount(value)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}