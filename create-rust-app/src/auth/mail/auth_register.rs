use crate::Mailer;

#[allow(dead_code)]
pub fn send(mailer: &Mailer, to_email: &str, link: &str) {
    let subject = "Registration Confirmation)";
    let text = format!(
        r#"
(This is an automated message.)

Hello,

Please follow the link below to complete your registration:
{link}
"#
    );
    let html = format!(
        r#"
(This is an automated message.)

Hello,

Please follow the link below to complete your registration:
{link}
"#
    );

    mailer.send(to_email, subject, &text, &html);
}
