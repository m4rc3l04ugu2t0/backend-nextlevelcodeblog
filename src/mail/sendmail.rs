use crate::{Result};
use lettre::{
    message::{header, SinglePart},
    transport::smtp::{authentication::Credentials},
    Message, SmtpTransport, Transport,
};
use tracing::info;
use std::{env::var, fs::read_to_string};

pub async fn send_email(
    to: &str,
    subject: &str,
    template_path: &str,
    placeholders: &[(String, String)],
) -> Result<()> {
    // Environment variable handling with better error messages
    let smtp_username = var("SMTP_USERNAME").expect("SMTP_USERNAME environment variable not set");
    let smtp_password = var("SMTP_PASSWORD").expect("SMTP_PASSWORD environment variable not set");
    let smtp_server = var("SMTP_SERVER").unwrap_or_else(|_| "smtp.gmail.com".to_string());
    let smtp_port: u16 = var("SMTP_PORT")
        .unwrap_or_else(|_| "587".to_string())
        .parse()
        .expect("Invalid SMTP_PORT format");

    // Security: Remove password logging
    info!("Attempting to send email via {}", smtp_server);

    // Template handling with error propagation
    let mut html_template = read_to_string(template_path)
        .map_err(|e| format!("Failed to read template: {}", e))?;

    for (k, v) in placeholders {
        html_template = html_template.replace(k, v);
    }

    // Email construction with proper error handling
    let email = Message::builder()
        .from(smtp_username.parse().map_err(|e| format!("Invalid from address: {}", e))?)
        .to(to.parse().map_err(|e| format!("Invalid to address: {}", e))?)
        .subject(subject)
        .header(header::ContentType::TEXT_HTML)
        .singlepart(
            SinglePart::builder()
                .header(header::ContentType::TEXT_HTML)
                .body(html_template),
        )
        .map_err(|e| format!("Email construction failed: {}", e))?;

    // SMTP configuration with conditional encryption
    let creds = Credentials::new(smtp_username.clone(), smtp_password.clone());

    let mailer = if smtp_port == 465 {
        // SSL connection
        SmtpTransport::relay(&smtp_server)
            .map_err(|e| format!("SMTP relay configuration failed: {}", e))?
            .credentials(creds)
            .port(smtp_port)
            .build()
    } else {
        // STARTTLS (default)
        SmtpTransport::starttls_relay(&smtp_server)
            .map_err(|e| format!("SMTP STARTTLS configuration failed: {}", e))?
            .credentials(creds)
            .port(smtp_port)
            .build()
    };

    // Async email sending with proper error handling
    let result = mailer.send(&email);

    match result {
        Ok(_) => println!("Email sent successfully!"),
        Err(e) => println!("Failed to send email: {:?}", e),
    }

    Ok(())
}