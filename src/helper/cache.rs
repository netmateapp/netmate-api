use http::HeaderValue;
use xxhash_rust::xxh3::xxh3_64;

pub fn check_if_none_match(byets: &[u8], if_none_match: &HeaderValue) -> bool {
    let hash = xxh3_64(byets);
    match if_none_match.to_str() {
        Ok(s) => match s.parse::<u64>() {
            Ok(v) => v == hash,
            Err(_) => false
        },
        Err(_) => false
    }
}

pub fn create_etag(bytes: &[u8]) -> HeaderValue {
    let hash = xxh3_64(bytes);
    HeaderValue::from_str(&hash.to_string()).unwrap()
}