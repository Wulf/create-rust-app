extern crate lettre;
extern crate lettre_email;

use lettre::smtp::authentication::Credentials;
use lettre::{SmtpClient, Transport};
use lettre::stub::StubTransport;
use lettre_email::EmailBuilder;

pub fn send() {
  let FROM_ADDRESS: String = std::env::var("SMTP_FROM_ADDRESS").unwrap();
  let SMTP_SERVER: String = std::env::var("SMTP_SERVER").unwrap();
  let SMTP_USERNAME: String = std::env::var("SMTP_USERNAME").unwrap();
  let SMTP_PASSWORD: String = std::env::var("SMTP_PASSWORD").unwrap();
  let ACTUALLY_SEND: bool = std::env::var("SEND_EMAIL").unwrap().eq("true");

  let to = "test@test.com";
  let from = FROM_ADDRESS;
  let subject = format!("Invitation)");
  let text = format!(r#"
Hello,

We hope you're having a good day. You have been invited to join the community!
Follow the link below to complete your profile:
{link}

Warmest regards
"#, 
    link="https://app.my-domain.com/register"
  );

  let email = EmailBuilder::new()
    .to(to.clone())
    .from(from)
    .subject(subject)
    .text(text)
    .build()
    .unwrap();
  
  if ACTUALLY_SEND {
    let mut mailer = SmtpClient::new_simple(SMTP_SERVER.as_str())
      .unwrap()
      .credentials(Credentials::new(SMTP_USERNAME.into(), SMTP_PASSWORD.into()))
      .transport();

    let result = mailer.send(email.into());
    println!("{:?}", result);
  } else {
    let mut mailer = StubTransport::new_positive();
    let result = mailer.send(email.into());
    println!("{:?}", result);
  }
}