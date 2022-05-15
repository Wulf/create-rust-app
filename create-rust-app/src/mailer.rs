use lettre::smtp::authentication::Credentials;
use lettre::stub::StubTransport;
use lettre::{SendableEmail, SmtpClient, Transport};
use lettre_email::EmailBuilder;

#[derive(Clone)]
pub struct Mailer {
    pub from_address: String,
    pub smtp_server: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub actually_send: bool,
}

impl Mailer {
    pub fn new() -> Mailer {
        Mailer::check_environment_variables();

        let from_address: String =
            std::env::var("SMTP_FROM_ADDRESS").unwrap_or("create-rust-app@localhost".to_string());
        let smtp_server: String = std::env::var("SMTP_SERVER").unwrap_or("".to_string());
        let smtp_username: String = std::env::var("SMTP_USERNAME").unwrap_or("".to_string());
        let smtp_password: String = std::env::var("SMTP_PASSWORD").unwrap_or("".to_string());
        let actually_send: bool = std::env::var("SEND_MAIL")
            .unwrap_or("false".to_string())
            .eq_ignore_ascii_case("true");

        return Mailer {
            from_address,
            smtp_server,
            smtp_username,
            smtp_password,
            actually_send,
        };
    }

    pub fn check_environment_variables() {
        if std::env::var("SMTP_FROM_ADDRESS").is_err() {
            println!(
                "Note: Mailing disabled; 'SMTP_FROM_ADDRESS' not set."
            );
        }

        if std::env::var("SMTP_SERVER").is_err() {
            println!("Note: Mailing disabled; 'SMTP_SERVER' not set.");
        }

        if std::env::var("SMTP_USERNAME").is_err() {
            println!("Note: Mailing disabled; 'SMTP_USERNAME' not set.");
        }

        if std::env::var("SMTP_PASSWORD").is_err() {
            println!("Note: Mailing disabled; 'SMTP_PASSWORD' not set.");
        }

        if std::env::var("SEND_MAIL").is_err() || !std::env::var("SEND_MAIL").unwrap().eq_ignore_ascii_case("true") {
            println!(
                "Note: Mailing disabled; 'SEND_MAIL' not 'true'."
            );
        }
    }

    pub fn send(&self, to: &str, subject: &str, text: &str, html: &str) {
        let email: SendableEmail = EmailBuilder::new()
            .to(to)
            .from(self.from_address.as_ref())
            .subject(subject)
            .text(text)
            .html(html)
            .build()
            .unwrap()
            .into();

        if self.actually_send {
            let mut mailer = SmtpClient::new_simple(self.smtp_server.as_str())
                .unwrap()
                .credentials(Credentials::new(
                    self.smtp_username.to_string(),
                    self.smtp_password.to_string(),
                ))
                .transport();

            let result = mailer.send(email);
            println!(
                r#"====================
Sent email {:#?}
--------------------
to: {:?}
from: {}
message:
{}
===================="#,
                result, to, self.from_address, text
            );
        } else {
            let mut mailer = StubTransport::new_positive();
            let result = mailer.send(email);
            println!(
                r#"====================
Sent email {:#?}
--------------------
to: {:?}
from: {}
message:
{}
===================="#,
                result, to, self.from_address, text
            );
        }
    }
}
