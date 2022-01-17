extern crate lettre;
extern crate lettre_email;
use crate::util::env;
use lettre::smtp::authentication::Credentials;
use lettre::stub::StubTransport;
use lettre::{SendableEmail, SmtpClient, Transport};
use lettre_email::EmailBuilder;
use std::str;

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

        let from_address: String = env::var("SMTP_FROM_ADDRESS", "create-rust-app@localhost");
        let smtp_server: String = env::var("SMTP_SERVER", "");
        let smtp_username: String = env::var("SMTP_USERNAME", "");
        let smtp_password: String = env::var("SMTP_PASSWORD", "");
        let actually_send: bool = env::var("SEND_MAIL", "false").eq("true");

        return Mailer {
            from_address: from_address,
            smtp_server: smtp_server,
            smtp_username: smtp_username,
            smtp_password: smtp_password,
            actually_send: actually_send,
        };
    }

    pub fn check_environment_variables() {
        if std::env::var("SECRET_KEY").is_err() {
            panic!("No SECRET_KEY environment variable set!");
        }

        if std::env::var("DATABASE_URL").is_err() {
            panic!("No DATABASE_URL environment variable set!");
        }

        if std::env::var("SMTP_FROM_ADDRESS").is_err() {
            println!(
                "Note: SMTP_FROM_ADDRESS environment variable is not set, no mail will be sent!"
            );
        }

        if std::env::var("SMTP_SERVER").is_err() {
            println!("Note: SMTP_SERVER environment variable is not set, no mail will be sent!");
        }

        if std::env::var("SMTP_USERNAME").is_err() {
            println!("Note: SMTP_USERNAME environment variable is not set, no mail will be sent!");
        }

        if std::env::var("SMTP_PASSWORD").is_err() {
            println!("Note: SMTP_PASSWORD environment variable is not set, no mail will be sent!");
        }

        if std::env::var("SEND_MAIL").is_err() || !std::env::var("SEND_MAIL").unwrap().eq("true") {
            println!(
                "Note: SEND_MAIL environment variable is not set to \"true\", no mail will be sent!"
            );
        }
    }

    pub fn send(&self, to: &str, subject: &str, text: &str, html: &str) {
        let email = EmailBuilder::new()
            .to(to)
            .from(self.from_address.as_ref())
            .subject(subject)
            .text(text)
            .html(html)
            .build()
            .unwrap();

        let sendable_email: SendableEmail = email.clone().into();

        if self.actually_send {
            let mut mailer = SmtpClient::new_simple(self.smtp_server.as_str())
                .unwrap()
                .credentials(Credentials::new(
                    self.smtp_username.to_string(),
                    self.smtp_password.to_string(),
                ))
                .transport();

            let result = mailer.send(sendable_email);
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
            let result = mailer.send(sendable_email);
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

pub mod invitation;
