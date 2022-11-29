use crate::Mailer;

#[allow(dead_code)]
pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Your password was changed)";
    let text = r#"
(This is an automated message.)

Hello,

Your password was changed successfully!
"#
    .to_string();
    let html = r#"
(This is an automated message.)

Hello,

Your password was changed successfully!
"#
    .to_string();

    mailer.send(to_email, subject, &text, &html);
}
