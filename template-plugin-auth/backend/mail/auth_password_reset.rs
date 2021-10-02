use crate::mail::Mailer;

pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Your password was reset)";
    let text = format!(
        r#"
(This is an automated message.)

Hello,

Your password was successfully reset!
"#
    );
    let html = format!(
        r#"
(This is an automated message.)

Hello,

Your password was successfully reset!
"#
    );

    mailer.send(to_email, &subject, &text, &html);
}
