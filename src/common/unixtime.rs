use std::time::{SystemTime, UNIX_EPOCH};

pub struct UnixtimeMillis(u64);

impl UnixtimeMillis {
    pub fn new(unixtime: u64) -> Self {
        Self(unixtime)
    }

    pub fn now() -> Self {
        // プログラム開始時に時刻の正常性を確認しているため、`unwrap()`で問題ない
        Self(SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64)
    }

    pub fn value(&self) -> u64 {
        self.0
    }
}