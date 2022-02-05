use crate::Mailer;

pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Your password was changed)";
    let text = format!(
        r#"
(This is an automated message.)

Hello,

Your password was changed successfully!
"#
    );
    let html = format!(
        r#"
(This is an automated message.)

Hello,

Your password was changed successfully!
"#
    );

    mailer.send(to_email, &subject, &text, &html);
}
