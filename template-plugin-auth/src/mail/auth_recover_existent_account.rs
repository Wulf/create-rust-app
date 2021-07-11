use crate::mail::Mailer;

pub fn send(mailer: &Mailer, to_email: &str, link: &str) {
  let subject = "Reset Password Instructions)";
  let text = format!(r#"
(This is an automated message.)

Hello,

Someone requested a password reset for the account associated with this email.
Please visit this link to reset your password:
{link}
(valid for 24 hours)
"#, link=link);
  let html = format!(r#"
(This is an automated message.)

Hello,

Someone requested a password reset for the account associated with this email.
Please visit this link to reset your password:
{link}
(valid for 24 hours)
"#, link=link);

  mailer.send(to_email, &subject, &text, &html);
}