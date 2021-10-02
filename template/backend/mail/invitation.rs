use crate::mail::Mailer;

pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Invitation";
    let text = format!(
        r#"
Hello,

We hope you're having a good day. You have been invited to visit our website!
Follow the link below to complete your profile:
{link}

Warmest regards
"#,
        link = "https://app.my-domain.com"
    );
    let html = format!(
        r#"
<h1>Hello,</h1>

<p>We hope you're having a good day. You have been invited to visit our website!
Follow the link below to complete your profile:
<a href="{link}">{link}</a></p>

<p>Warmest regards</p>
"#,
        link = "https://app.my-domain.com"
    );

    mailer.send(to_email, &subject, &text, &html);
}
