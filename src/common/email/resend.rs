use resend_rs::{types::CreateEmailBaseOptions, Resend};

use super::{address::Email, send::{Body, EmailSendFailed, EmailSender, NetmateEmail, SenderName, Subject}};

pub struct ResendEmailService;

impl EmailSender for ResendEmailService {
    async fn send(from: &NetmateEmail, to: &Email, sender_name: &SenderName, subject: &Subject, body: &Body) -> Result<(), EmailSendFailed> {
        let resend = Resend::new("");

        let from = format!("{} <{}>", sender_name.value(), from.value().value());
        let to = [to.value()];

        let email = CreateEmailBaseOptions::new(from, to, subject.value())
            .with_html(&body.html_content().value())
            .with_text(&body.plain_text().value());

        resend.emails
            .send(email)
            .await
            .map(|_| ())
            .map_err(|e| EmailSendFailed(e.into()))
    }
}