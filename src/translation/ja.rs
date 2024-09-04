pub mod email {
    pub const SENDER_NAME: &str = "ネットメイト";
}

pub mod session {
    pub const SECURITY_NOTIFICATION_SUBJECT: &str = "アカウントが不正に利用されている恐れがあります";
    pub const SECURITY_NOTIFICATION_BODY_HTML: &str = concat!(
        "<p>アカウントが不正に利用されている恐れがあります。</p>",
        "<p>応急措置として全ての端末がログアウトされました。</p>",
        "<p>パスワードを再設定し、アカウントの状態を確認してください。</p>",
        "<p>https://netmate.app/recovery-account</p>",
    );
    pub const SECURITY_NOTIFICATION_BODY_PLAIN: &str = concat!(
        "アカウントが不正に利用されている恐れがあるため、アカウントのセキュリティを確認してください。",
        "https://netmate.app/security",
    );
}

pub mod sign_up {
    pub const AUTHENTICATION_EMAIL_SUBJECT: &str = "メールアドレスの認証をしてください";
    pub const ATUHENTICATION_EMAIL_BODY_HTML: &str = concat!(
        "<p>次のリンクをクリックし、メールアドレスを認証を完了してください。</p>",
        "<p><a href=\"https://netmate.app/verify-email/{token}\">https://netmate.app/verify-email/{token}</a></p>",
    );
    pub const AUTHENTICATION_EMAIL_BODY_PLAIN: &str = concat!(
        "次のリンクをクリックし、メールアドレスを認証を完了してください。",
        "https://netmate.app/verify-email/{token}",
    );
}