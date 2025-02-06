use crate::Result;
use lettre::{message::{header, SinglePart}, transport::smtp::authentication::Credentials, Message, SmtpTransport, Transport};
use std::{env::var, fs::read_to_string};

pub async fn send_email(
    to: &str,
    subject: &str,
    template_path: &str,
    placeholders: &[(String, String)],
) -> Result<()> {
    let smtp_username = var("SMTP_USERNAME").expect("SMTP_USERNAME must be set");
    let smtp_password = var("SMTP_PASSWORD").expect("SMTP_PASSWORD must be set");
    let smtp_server = var("SMTP_SERVER").expect("SMTP_SERVER must be set");
    let smtp_port: u16 = var("SMTP_PORT")
        .expect("SMPT_PORT must be set")
        .parse()
        .unwrap();

    let mut html_template = read_to_string(template_path).unwrap();

    for (k, v) in placeholders {
        html_template = html_template.replace(k, v);
    }

    let email = Message::builder()
        .from(smtp_username.parse().unwrap())
        .to(to.parse().unwrap())
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .singlepart(SinglePart::builder().header(header::ContentType::TEXT_HTML).body(html_template))
        .unwrap();

         let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());
    let mailer = SmtpTransport::starttls_relay(&smtp_server).unwrap()
        .credentials(creds)
        .port(smtp_port)
        .build();

    let result = mailer.send(&email);

    match result {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => println!("Failed to send email: {:?}", e),
    }

    Ok(())
}
