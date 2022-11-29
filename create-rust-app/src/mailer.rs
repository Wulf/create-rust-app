use lettre::message::{Message, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::stub::StubTransport;
use lettre::{SmtpTransport, Transport};

#[derive(Clone)]
pub struct Mailer {
    pub from_address: String,
    pub smtp_server: String,
    pub smtp_username: String,
    pub smtp_password: String,
    pub actually_send: bool,
}

impl Default for Mailer {
    fn default() -> Self {
        Self::new()
    }
}

impl Mailer {
    pub fn new() -> Self {
        Mailer::check_environment_variables();

        let from_address: String = std::env::var("SMTP_FROM_ADDRESS")
            .unwrap_or_else(|_| "create-rust-app@localhost".to_string());
        let smtp_server: String = std::env::var("SMTP_SERVER").unwrap_or_else(|_| "".to_string());
        let smtp_username: String =
            std::env::var("SMTP_USERNAME").unwrap_or_else(|_| "".to_string());
        let smtp_password: String =
            std::env::var("SMTP_PASSWORD").unwrap_or_else(|_| "".to_string());
        let actually_send: bool = std::env::var("SEND_MAIL")
            .unwrap_or_else(|_| "false".to_string())
            .eq_ignore_ascii_case("true");

        Mailer {
            from_address,
            smtp_server,
            smtp_username,
            smtp_password,
            actually_send,
        }
    }

    pub fn check_environment_variables() {
        if std::env::var("SMTP_FROM_ADDRESS").is_err() {
            println!("Note: Mailing disabled; 'SMTP_FROM_ADDRESS' not set.");
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

        if std::env::var("SEND_MAIL").is_err()
            || !std::env::var("SEND_MAIL")
                .unwrap()
                .eq_ignore_ascii_case("true")
        {
            println!("Note: Mailing disabled; 'SEND_MAIL' not 'true'.");
        }
    }

    pub fn send(&self, to: &str, subject: &str, text: &str, html: &str) {
        let email = Message::builder()
            .to(to.parse().unwrap())
            .from(self.from_address.parse().unwrap())
            .subject(subject)
            .multipart(MultiPart::alternative_plain_html(
                String::from(text),
                String::from(html),
            ))
            .unwrap();

        if self.actually_send {
            let mailer = SmtpTransport::relay(&self.smtp_server)
                .unwrap()
                .credentials(Credentials::new(
                    self.smtp_username.to_string(),
                    self.smtp_password.to_string(),
                ))
                .build();

            let result = mailer.send(&email);
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
            let mailer = StubTransport::new_ok();
            let result = mailer.send(&email);
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
