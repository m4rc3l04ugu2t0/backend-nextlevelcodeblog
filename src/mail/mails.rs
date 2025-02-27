use std::env;

use crate::Result;

use super::sendmail::send_email;

pub async fn send_verification_email(to_email: &str, username: &str, token: &str) -> Result<()> {
    let subject = "Email Verification";
    let template_path = "src/mail/templates/Verification-email.html";
    let base_url = &format!(
        "{}/confirm-auth/verify-email",
        env::var("FRONT_URL").expect("FRONT_URL must be set")
    );
    let verification_link = create_verification_link(base_url, token);
    let login_url = env::var("FRONT_URL").expect("FRONT_URL must be set");
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{verification_link}}".to_string(), verification_link),
        ("{{login_url}}".to_string(), login_url),
    ];

    send_email(to_email, subject, template_path, &placeholders).await
}

fn create_verification_link(base_url: &str, token: &str) -> String {
    format!("{}?token={}", base_url, token)
}

pub async fn send_welcome_email(to_email: &str, username: &str) -> Result<()> {
    let subject = "Welcome to Application";
    let template_path = "src/mail/templates/Welcome-email.html";
    let x_url = "x.com/next_level_code".to_string();
    let github_url = "https://github.com/m4rc3l04ugu2t0".to_string();
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{x_url}}".to_string(), x_url.to_string()),
        ("{{github_url}}".to_string(), github_url.to_string()),
    ];

    send_email(to_email, subject, template_path, &placeholders).await
}

pub async fn send_forgot_password_email(
    to_email: &str,
    reset_link: &str,
    username: &str,
) -> Result<()> {
    let subject = "Rest your Password";
    let template_path = "src/mail/templates/RestPassword-email.html";
    let placeholders = vec![
        ("{{username}}".to_string(), username.to_string()),
        ("{{reset_link}}".to_string(), reset_link.to_string()),
    ];

    send_email(to_email, subject, template_path, &placeholders).await
}
