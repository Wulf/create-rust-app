use crate::Mailer;

#[allow(dead_code)]
pub fn send(mailer: &Mailer, to_email: &str, link: &str) {
    let subject = "Reset Password Instructions";
    let text = format!(
        r#"
(This is an automated message.)

Hello,

Someone requested a password reset for the account associated with this email, but no account exists!
If this was intentional, you can register for a new account using the link below:
{link}
"#
    );
    let html = format!(
        r#"
<p>(This is an automated message.)</p>

<p>Hello,<p>

<p>Someone requested a password reset for the account associated with this email, but no account exists!
If this was intentional, you can register for a new account using the link below:</p>
<p><a href="{link}">{link}</a></p>
"#
    );

    mailer.send(to_email, subject, &text, &html);
}
