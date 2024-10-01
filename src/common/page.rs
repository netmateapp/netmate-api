#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct ZeroBasedPage(u32);

impl ZeroBasedPage {
    pub fn value(&self) -> u32 {
        self.0
    }

    pub fn first_index(&self, page_size: u32) -> u32 {
        self.0 * page_size
    }

    pub fn last_index(&self, page_size: u32) -> u32 {
        self.first_index(page_size) + page_size - 1
    }
}