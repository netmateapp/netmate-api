use resend_rs::{types::CreateEmailBaseOptions, Resend};

use super::{address::Email, send::{Body, EmailSendFailed, EmailSender, NetmateEmail, SenderName, Subject}};

pub struct ResendEmailSender;

impl EmailSender for ResendEmailSender {
    async fn send(from: &NetmateEmail, to: &Email, sender_name: &SenderName, subject: &Subject, body: &Body) -> Result<(), EmailSendFailed> {
        let resend = Resend::new("");

        // ネットメイト <example@netmate.app>
        let from = format!("{} <{}>", sender_name, from);
        let to = [to.value()];

        let email = CreateEmailBaseOptions::new(from, to, subject.to_string())
            .with_html(&body.html_content().to_string())
            .with_text(&body.plain_text().to_string());

        resend.emails
            .send(email)
            .await
            .map(|_| ())
            .map_err(|e| EmailSendFailed(e.into()))
    }
}