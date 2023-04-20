use crate::Mailer;

#[allow(dead_code)]
pub fn send(mailer: &Mailer, to_email: &str, link: &str) {
    let subject = "Registration Confirmation";
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
<p>(This is an automated message.)</p>
<br>
<p>Hello,</p>
<br>
<p>Please follow the link below to complete your registration:</p>
<p><a href="{link}">{link}</a></p>
"#
    );

    mailer.send(to_email, subject, &text, &html);
}
