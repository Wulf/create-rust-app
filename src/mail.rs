use indoc::indoc;
use anyhow::Result;

const MAIL_FILE_TEMPLATE: &str = indoc! {r##"
use crate::mail::Mailer;

pub fn send(mailer: &Mailer, to_email: &str$MAIL_VARIABLES) {
  let subject = "$MAIL_SUBJECT";
  let text = format!(r#"$MAIL_TEXT_BODY"#, $MAIL_INJECT_VARIABLES);
  let html = format!(r#"$MAIL_HTML_BODY"#, $MAIL_INJECT_VARIABLES);
  
  mailer.send(to_email, &subject, &text, &html);
}
"##};

pub fn create(file_name: &str, vars: Vec<&str>, subject: &str, text: &str, html: &str) -> Result<()> {
  let contents = MAIL_FILE_TEMPLATE
    .replace("$MAIL_SUBJECT", subject)
    .replace("$MAIL_TEXT_BODY", text)
    .replace("$MAIL_HTML_BODY", html)
    .replace("$MAIL_VARIABLES", (&vars).into_iter().map(|v| format!(", {}: &str", v)).collect::<Vec<String>>().join("").as_str())
    .replace("$MAIL_INJECT_VARIABLES", vars.join(", ").as_str());

  crate::fs::add_rust_file(
    "backend/mail",
    file_name,
    contents.as_str()
  )?;

  Ok(())
}

