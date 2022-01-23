use create_rust_app::Mailer;

pub fn send(mailer: &Mailer, to_email: &str) {
    let subject = "Example Email";
    let text = format!(
        r#"
Hello,

We hope you're having a good day. You have been invited to visit our website!

{link}

Warmest regards
"#,
        link = "https://app.my-domain.com"
    );
    let html = format!(
        r#"
<h1>Hello,</h1>

<p>We hope you're having a good day. You have been invited to visit our website!

<a href="{link}">{link}</a></p>

<p>Warmest regards</p>
"#,
        link = "https://app.my-domain.com"
    );

    mailer.send(to_email, &subject, &text, &html);
}
