use crate::Mailer;

#[allow(dead_code)]
pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Account activated";
    let text = r#"
(This is an automated message.)

Hello,

Your account has been activated!
"#
    .to_string();
    let html = r#"
<p>(This is an automated message.)</p>

<p>Hello,</p>

<p>Your account has been activated!</p>
"#
    .to_string();

    mailer.send(to_email, subject, &text, &html);
}
