use lettre::message::{Message, MultiPart};
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::stub::StubTransport;
use lettre::{SmtpTransport, Transport};

#[derive(Clone)]
/// struct used to handle sending emails
pub struct Mailer {
    /// the email address emails should be sent from
    ///
    /// set by the `SMTP_FROM_ADDRESS` environment variable
    pub from_address: String,
    /// the smtp server to connect to for purposes of sending emails
    ///
    /// set by the `SMTP_SERVER` environment variable
    pub smtp_server: String,
    /// username used to log into `SMTP_SERVER`
    ///
    /// set by the `SMTP_USERNAME` environment variable
    pub smtp_username: String,
    /// the password used to log into `SMTP_SERVER`
    ///
    /// set by the `SMTP_PASSWORD' environment variable
    pub smtp_password: String,
    /// whether or not emails should actually be sent when requested
    ///
    /// it may be useful to set this to false in some devolopment environments
    /// while setting it to true in production
    ///
    /// set by the `SEND_MAIL` environment variable
    pub actually_send: bool,
}

impl Mailer {
    /// using information stored in the `SMTP_FROM_ADDRESS`, `SMTP_SERVER`, `SMTP_USERNAME`, `SMTP_PASSWORD`, and `SEND_MAIL`
    /// environment variables to connect to a remote SMTP server,
    ///
    /// allows webservers to send emails to users for purposes
    /// like marketing, user authentification, etc.
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

    /// checks that the required environment variables are set
    ///
    /// prints messages denoting which, if any, of the required
    /// environment variables were not set
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

    /// send an email with the specifified content and subject to the specified user
    ///
    /// will only send an email if the `SEND_MAIL` environment variable was set to true when
    /// this mailer was initialized.
    ///
    /// # Arguments
    /// * `to` - a string slice that holds the email address of the intended recipient
    /// * `subject` - subject field of the email
    /// * `text` - text content of the email
    /// * `html` - html content of the email
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
