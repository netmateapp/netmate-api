pub mod email {
    pub const SENDER_NAME: &str = "ネットメイト";
}

pub mod sign_up {
    pub const AUTHENTICATION_EMAIL_SUBJECT: &str = "メールアドレスの認証をしてください。";
    pub const ATUHENTICATION_EMAIL_BODY_HTML: &str = concat!(
        "<p>次のリンクをクリックし、メールアドレスを認証を完了してください。</p>",
        "<p><a href=\"https://netmate.app/verify-email/{token}\">https://netmate.app/verify-email/{token}</a></p>",
    );
    pub const AUTHENTICATION_EMAIL_BODY_PLAIN: &str = concat!(
        "次のリンクをクリックし、メールアドレスを認証を完了してください。",
        "https://netmate.app/verify-email/{token}",
    );
}