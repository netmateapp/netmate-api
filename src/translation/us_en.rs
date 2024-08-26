pub mod email {
    pub const SENDER_NAME: &str = "Netmate";
}

pub mod sign_up {
    pub const AUTHENTICATION_EMAIL_SUBJECT: &str = "Please verify your email address.";
    pub const ATUHENTICATION_EMAIL_BODY_HTML: &str = concat!(
        "<p>Please click the following link to complete the verification of your email address.</p>",
        "<p><a href=\"https://netmate.app/verify-email/{token}\">https://netmate.app/verify-email/{token}</a></p>"
    );
    pub const AUTHENTICATION_EMAIL_BODY_PLAIN: &str = concat!(
        "Please click the following link to complete the verification of your email address.",
        "https://netmate.app/verify-email/{token}",
    );
}