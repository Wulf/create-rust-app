use crate::Mailer;

pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Account activated)";
    let text = format!(
        r#"
(This is an automated message.)

Hello,

Your account has been activated!
"#
    );
    let html = format!(
        r#"
(This is an automated message.)

Hello,

Your account has been activated!
"#
    );

    mailer.send(to_email, &subject, &text, &html);
}
