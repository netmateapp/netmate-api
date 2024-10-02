pub struct TurnstileToken(String);

impl TurnstileToken {
    pub fn new(token: String) -> Self {
        Self(token)
    }

    pub fn value(&self) -> &String {
        &self.0
    }
}