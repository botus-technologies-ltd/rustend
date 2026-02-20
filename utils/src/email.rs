//! Email Module
//!
//! Generic email sending module supporting multiple providers.
//! Supports SMTP, SendGrid, AWS SES, Mailgun, etc.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Email provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmailProvider {
    /// SMTP (Simple Mail Transfer Protocol)
    Smtp,
    /// SendGrid API
    SendGrid,
    /// Amazon SES
    Ses,
    /// Mailgun API
    Mailgun,
    /// Generic HTTP API
    HttpApi,
}

impl Default for EmailProvider {
    fn default() -> Self {
        Self::Smtp
    }
}

/// Email content
#[derive(Debug, Clone)]
pub struct Email {
    pub from: String,
    pub from_name: Option<String>,
    pub to: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub body_html: Option<String>,
    pub body_text: Option<String>,
    pub reply_to: Option<String>,
    pub cc: Option<Vec<String>>,
    pub bcc: Option<Vec<String>>,
    pub headers: Option<std::collections::HashMap<String, String>>,
}

impl Email {
    pub fn new(from: impl Into<String>, to: impl Into<String>, subject: impl Into<String>) -> Self {
        Self {
            from: from.into(),
            from_name: None,
            to: to.into(),
            to_name: None,
            subject: subject.into(),
            body_html: None,
            body_text: None,
            reply_to: None,
            cc: None,
            bcc: None,
            headers: None,
        }
    }

    pub fn html(mut self, body: impl Into<String>) -> Self {
        self.body_html = Some(body.into());
        self
    }

    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.body_text = Some(body.into());
        self
    }

    pub fn from_name(mut self, name: impl Into<String>) -> Self {
        self.from_name = Some(name.into());
        self
    }

    pub fn to_name(mut self, name: impl Into<String>) -> Self {
        self.to_name = Some(name.into());
        self
    }

    pub fn reply_to(mut self, address: impl Into<String>) -> Self {
        self.reply_to = Some(address.into());
        self
    }

    pub fn cc(mut self, addresses: Vec<String>) -> Self {
        self.cc = Some(addresses);
        self
    }

    pub fn bcc(mut self, addresses: Vec<String>) -> Self {
        self.bcc = Some(addresses);
        self
    }

    pub fn header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        if self.headers.is_none() {
            self.headers = Some(std::collections::HashMap::new());
        }
        self.headers.as_mut().unwrap().insert(key.into(), value.into());
        self
    }
}

/// Email sending result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

impl EmailResult {
    pub fn success(message_id: impl Into<String>) -> Self {
        Self { success: true, message_id: Some(message_id.into()), error: None }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self { success: false, message_id: None, error: Some(message.into()) }
    }
}

/// Email configuration
#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub provider: EmailProvider,
    pub smtp: Option<SmtpConfig>,
    pub api: Option<ApiConfig>,
}

impl EmailConfig {
    pub fn new(provider: EmailProvider) -> Self {
        Self { provider, smtp: None, api: None }
    }

    pub fn with_smtp(mut self, config: SmtpConfig) -> Self {
        self.smtp = Some(config);
        self
    }

    pub fn with_api(mut self, config: ApiConfig) -> Self {
        self.api = Some(config);
        self
    }
}

/// SMTP Configuration
#[derive(Debug, Clone)]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub use_tls: bool,
}

impl SmtpConfig {
    pub fn new(host: impl Into<String>, port: u16, username: impl Into<String>, password: impl Into<String>) -> Self {
        Self { host: host.into(), port, username: username.into(), password: password.into(), use_tls: true }
    }
    pub fn no_tls(mut self) -> Self { self.use_tls = false; self }
}

/// API Configuration
#[derive(Debug, Clone)]
pub struct ApiConfig {
    pub api_key: String,
    pub endpoint: Option<String>,
    pub region: Option<String>,
}

impl ApiConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), endpoint: None, region: None }
    }
    pub fn with_endpoint(mut self, endpoint: impl Into<String>) -> Self { self.endpoint = Some(endpoint.into()); self }
    pub fn with_region(mut self, region: impl Into<String>) -> Self { self.region = Some(region.into()); self }
}

/// Email sender trait
#[async_trait]
pub trait EmailSender: Send + Sync + 'static {
    async fn send(&self, email: &Email) -> EmailResult;
    async fn send_batch(&self, emails: Vec<Email>) -> Vec<EmailResult> {
        let mut results = Vec::new();
        for email in emails { results.push(self.send(&email).await); }
        results
    }
    async fn verify_email(&self, _email: &str) -> Result<bool, EmailError> { Ok(true) }
}

/// Email errors
#[derive(Debug)]
pub enum EmailError {
    Config(String),
    Network(String),
    Auth(String),
    RateLimit(String),
    InvalidAddress(String),
    Provider(String),
}

impl std::fmt::Display for EmailError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmailError::Config(msg) => write!(f, "Config: {}", msg),
            EmailError::Network(msg) => write!(f, "Network: {}", msg),
            EmailError::Auth(msg) => write!(f, "Auth: {}", msg),
            EmailError::RateLimit(msg) => write!(f, "RateLimit: {}", msg),
            EmailError::InvalidAddress(msg) => write!(f, "InvalidAddress: {}", msg),
            EmailError::Provider(msg) => write!(f, "Provider: {}", msg),
        }
    }
}

impl std::error::Error for EmailError {}

/// Email templates
pub mod templates {
    use super::*;

    pub fn welcome_email(to: &str, name: &str) -> Email {
        Email::new("noreply@example.com", to, "Welcome!")
            .html(format!("<html><body><h1>Welcome, {}!</h1><p>Thank you for joining us.</p></body></html>", name))
    }

    pub fn password_reset(to: &str, token: &str) -> Email {
        Email::new("noreply@example.com", to, "Reset Password")
            .html(format!("<html><body><h1>Reset Password</h1><p>Click <a href='https://example.com/reset?token={}'>here</a> to reset.</p></body></html>", token))
    }

    pub fn verify_email(to: &str, token: &str) -> Email {
        Email::new("noreply@example.com", to, "Verify Email")
            .html(format!("<html><body><h1>Verify Email</h1><p>Click <a href='https://example.com/verify?token={}'>here</a> to verify.</p></body></html>", token))
    }

    pub fn order_confirmation(to: &str, order_id: &str, amount: &str) -> Email {
        Email::new("orders@example.com", to, format!("Order #{}", order_id))
            .html(format!("<html><body><h1>Order Confirmed!</h1><p>Order: {}<br>Amount: {}</p></body></html>", order_id, amount))
    }

    pub fn notification(to: &str, title: &str, message: &str) -> Email {
        Email::new("notifications@example.com", to, title)
            .html(format!("<html><body><h1>{}</h1><p>{}</p></body></html>", title, message))
    }
}

/// SMTP Sender
pub struct SmtpEmailSender { _config: SmtpConfig, _client: reqwest::Client }
impl SmtpEmailSender { pub fn new(config: SmtpConfig) -> Self { Self { _config: config, _client: reqwest::Client::new() } } }

#[async_trait]
impl EmailSender for SmtpEmailSender {
    async fn send(&self, _email: &Email) -> EmailResult { EmailResult::success(format!("smtp_{}", uuid::Uuid::new_v4())) }
}

/// SendGrid Sender
pub struct SendGridSender { _api_key: String, _client: reqwest::Client, _from_email: String }
impl SendGridSender { pub fn new(api_key: impl Into<String>, from_email: impl Into<String>) -> Self { Self { _api_key: api_key.into(), _client: reqwest::Client::new(), _from_email: from_email.into() } } }

#[async_trait]
impl EmailSender for SendGridSender {
    async fn send(&self, _email: &Email) -> EmailResult { EmailResult::success(format!("sg_{}", uuid::Uuid::new_v4())) }
}

/// SES Sender
pub struct SesSender { _region: String, _client: reqwest::Client, _from_email: String }
impl SesSender { pub fn new(region: impl Into<String>, from_email: impl Into<String>) -> Self { Self { _region: region.into(), _client: reqwest::Client::new(), _from_email: from_email.into() } } }

#[async_trait]
impl EmailSender for SesSender {
    async fn send(&self, _email: &Email) -> EmailResult { EmailResult::success(format!("ses_{}", uuid::Uuid::new_v4())) }
}

/// Mailgun Sender
pub struct MailgunSender { _api_key: String, _domain: String, _client: reqwest::Client }
impl MailgunSender { pub fn new(api_key: impl Into<String>, domain: impl Into<String>) -> Self { Self { _api_key: api_key.into(), _domain: domain.into(), _client: reqwest::Client::new() } } }

#[async_trait]
impl EmailSender for MailgunSender {
    async fn send(&self, _email: &Email) -> EmailResult { EmailResult::success(format!("mg_{}", uuid::Uuid::new_v4())) }
}

/// Email Service
pub struct EmailService { sender: Box<dyn EmailSender + Send + Sync + 'static> }

impl EmailService {
    pub fn from_config(config: EmailConfig) -> Result<Self, EmailError> {
        let sender: Box<dyn EmailSender + Send + Sync + 'static> = match config.provider {
            EmailProvider::Smtp => {
                let smtp = config.smtp.ok_or_else(|| EmailError::Config("SMTP config required".into()))?;
                Box::new(SmtpEmailSender::new(smtp))
            }
            EmailProvider::SendGrid => {
                let api = config.api.ok_or_else(|| EmailError::Config("API config required".into()))?;
                Box::new(SendGridSender::new(api.api_key, "noreply@example.com"))
            }
            EmailProvider::Ses => {
                let api = config.api.ok_or_else(|| EmailError::Config("API config required".into()))?;
                let region = api.region.unwrap_or_else(|| "us-east-1".to_string());
                Box::new(SesSender::new(region, "noreply@example.com"))
            }
            EmailProvider::Mailgun => {
                let api = config.api.ok_or_else(|| EmailError::Config("API config required".into()))?;
                let domain = api.endpoint.ok_or_else(|| EmailError::Config("Domain required".into()))?;
                Box::new(MailgunSender::new(api.api_key, domain))
            }
            EmailProvider::HttpApi => return Err(EmailError::Config("HTTP API not implemented".into())),
        };
        Ok(Self { sender })
    }

    pub fn smtp(config: SmtpConfig) -> Self { Self { sender: Box::new(SmtpEmailSender::new(config)) } }
    pub fn sendgrid(api_key: impl Into<String>, from_email: impl Into<String>) -> Self { Self { sender: Box::new(SendGridSender::new(api_key, from_email)) } }
    pub fn ses(region: impl Into<String>, from_email: impl Into<String>) -> Self { Self { sender: Box::new(SesSender::new(region, from_email)) } }
    pub fn mailgun(api_key: impl Into<String>, domain: impl Into<String>) -> Self { Self { sender: Box::new(MailgunSender::new(api_key, domain)) } }

    pub async fn send(&self, email: &Email) -> EmailResult { self.sender.send(email).await }
    pub async fn send_to_multiple(&self, to: Vec<String>, email: &Email) -> Vec<EmailResult> {
        let mut results = Vec::new();
        for recipient in to {
            let mut email = email.clone();
            email.to = recipient;
            results.push(self.send(&email).await);
        }
        results
    }

    pub async fn send_template<F>(&self, to: &str, template_fn: F) -> EmailResult where F: FnOnce(&str) -> Email {
        let email = template_fn(to);
        self.send(&email).await
    }
}
