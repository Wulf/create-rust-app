use crate::Mailer;

#[allow(dead_code)]
pub fn send(mailer: &Mailer, to_email: &str, link: &str) {
    let subject = "Reset Password Instructions";
    let text = format!(
        r#"
(This is an automated message.)

Hello,

Someone requested a password reset for the account associated with this email.
Please visit this link to reset your password:
{link}
(valid for 24 hours)
"#
    );
    let html = format!(
        r#"
<p>(This is an automated message.)</p>

<p>Hello,</p>

<p>Someone requested a password reset for the account associated with this email.
Please visit this link to reset your password:</p>
<p><a href="{link}">{link}</a></p>
<p>(valid for 24 hours)</p>
"#
    );

    mailer.send(to_email, subject, &text, &html);
}
