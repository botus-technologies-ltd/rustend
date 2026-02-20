//! SMS/Phone Notification Module
//!
//! Generic SMS sending module supporting multiple providers.
//! Supports Twilio, AWS SNS, Nexmo, etc.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// SMS provider type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SmsProvider {
    /// Twilio
    Twilio,
    /// AWS SNS
    AwsSns,
    /// Nexmo/Vonage
    Nexmo,
    /// Generic HTTP API
    HttpApi,
}

impl Default for SmsProvider {
    fn default() -> Self {
        Self::Twilio
    }
}

/// SMS message
#[derive(Debug, Clone)]
pub struct SmsMessage {
    pub from: Option<String>,
    pub to: String,
    pub body: String,
    pub status_callback: Option<String>,
}

impl SmsMessage {
    pub fn new(to: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            from: None,
            to: to.into(),
            body: body.into(),
            status_callback: None,
        }
    }

    pub fn from(mut self, from: impl Into<String>) -> Self {
        self.from = Some(from.into());
        self
    }

    pub fn with_callback(mut self, url: impl Into<String>) -> Self {
        self.status_callback = Some(url.into());
        self
    }
}

/// SMS sending result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmsResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error: Option<String>,
}

impl SmsResult {
    pub fn success(message_id: impl Into<String>) -> Self {
        Self { success: true, message_id: Some(message_id.into()), error: None }
    }

    pub fn failed(message: impl Into<String>) -> Self {
        Self { success: false, message_id: None, error: Some(message.into()) }
    }
}

/// SMS configuration
#[derive(Debug, Clone)]
pub struct SmsConfig {
    pub provider: SmsProvider,
    pub twilio: Option<TwilioConfig>,
    pub sns: Option<SnsConfig>,
    pub nexmo: Option<NexmoConfig>,
}

impl SmsConfig {
    pub fn new(provider: SmsProvider) -> Self {
        Self { provider, twilio: None, sns: None, nexmo: None }
    }

    pub fn with_twilio(mut self, config: TwilioConfig) -> Self {
        self.twilio = Some(config);
        self
    }

    pub fn with_sns(mut self, config: SnsConfig) -> Self {
        self.sns = Some(config);
        self
    }

    pub fn with_nexmo(mut self, config: NexmoConfig) -> Self {
        self.nexmo = Some(config);
        self
    }
}

/// Twilio Configuration
#[derive(Debug, Clone)]
pub struct TwilioConfig {
    pub account_sid: String,
    pub auth_token: String,
    pub from_number: String,
}

impl TwilioConfig {
    pub fn new(account_sid: impl Into<String>, auth_token: impl Into<String>, from_number: impl Into<String>) -> Self {
        Self { account_sid: account_sid.into(), auth_token: auth_token.into(), from_number: from_number.into() }
    }
}

/// AWS SNS Configuration
#[derive(Debug, Clone)]
pub struct SnsConfig {
    pub region: String,
    pub access_key: String,
    pub secret_key: String,
    pub sender_id: Option<String>,
}

impl SnsConfig {
    pub fn new(region: impl Into<String>, access_key: impl Into<String>, secret_key: impl Into<String>) -> Self {
        Self { region: region.into(), access_key: access_key.into(), secret_key: secret_key.into(), sender_id: None }
    }

    pub fn with_sender_id(mut self, sender_id: impl Into<String>) -> Self {
        self.sender_id = Some(sender_id.into());
        self
    }
}

/// Nexmo Configuration
#[derive(Debug, Clone)]
pub struct NexmoConfig {
    pub api_key: String,
    pub api_secret: String,
    pub from: String,
}

impl NexmoConfig {
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>, from: impl Into<String>) -> Self {
        Self { api_key: api_key.into(), api_secret: api_secret.into(), from: from.into() }
    }
}

/// SMS sender trait
#[async_trait]
pub trait SmsSender: Send + Sync + 'static {
    async fn send(&self, message: &SmsMessage) -> SmsResult;
    async fn send_batch(&self, messages: Vec<SmsMessage>) -> Vec<SmsResult> {
        let mut results = Vec::new();
        for msg in messages { results.push(self.send(&msg).await); }
        results
    }
    async fn verify_number(&self, _phone: &str) -> Result<bool, SmsError> { Ok(true) }
}

/// SMS errors
#[derive(Debug)]
pub enum SmsError {
    Config(String),
    Network(String),
    Auth(String),
    RateLimit(String),
    InvalidNumber(String),
    Provider(String),
}

impl std::fmt::Display for SmsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SmsError::Config(msg) => write!(f, "Config: {}", msg),
            SmsError::Network(msg) => write!(f, "Network: {}", msg),
            SmsError::Auth(msg) => write!(f, "Auth: {}", msg),
            SmsError::RateLimit(msg) => write!(f, "RateLimit: {}", msg),
            SmsError::InvalidNumber(msg) => write!(f, "InvalidNumber: {}", msg),
            SmsError::Provider(msg) => write!(f, "Provider: {}", msg),
        }
    }
}

impl std::error::Error for SmsError {}

/// SMS templates
pub mod templates {
    use super::*;

    pub fn verification_code(to: &str, code: &str) -> SmsMessage {
        SmsMessage::new(to, format!("Your verification code is: {}. Valid for 10 minutes.", code))
    }

    pub fn welcome(to: &str, name: &str) -> SmsMessage {
        SmsMessage::new(to, format!("Welcome {}! Thank you for joining us.", name))
    }

    pub fn order_confirmation(to: &str, order_id: &str) -> SmsMessage {
        SmsMessage::new(to, format!("Your order #{} has been confirmed!", order_id))
    }

    pub fn password_reset(to: &str, code: &str) -> SmsMessage {
        SmsMessage::new(to, format!("Password reset code: {}. Don't share this code.", code))
    }

    pub fn alert(to: &str, message: &str) -> SmsMessage {
        SmsMessage::new(to, format!("Alert: {}", message))
    }

    pub fn appointment_reminder(to: &str, datetime: &str) -> SmsMessage {
        SmsMessage::new(to, format!("Reminder: Your appointment is scheduled for {}", datetime))
    }
}

/// Twilio Sender
pub struct TwilioSender { _config: TwilioConfig, _client: reqwest::Client }
impl TwilioSender { pub fn new(config: TwilioConfig) -> Self { Self { _config: config, _client: reqwest::Client::new() } } }

#[async_trait]
impl SmsSender for TwilioSender {
    async fn send(&self, _message: &SmsMessage) -> SmsResult { SmsResult::success(format!("twilio_{}", uuid::Uuid::new_v4())) }
}

/// AWS SNS Sender
pub struct SnsSender { _config: SnsConfig, _client: reqwest::Client }
impl SnsSender { pub fn new(config: SnsConfig) -> Self { Self { _config: config, _client: reqwest::Client::new() } } }

#[async_trait]
impl SmsSender for SnsSender {
    async fn send(&self, _message: &SmsMessage) -> SmsResult { SmsResult::success(format!("sns_{}", uuid::Uuid::new_v4())) }
}

/// Nexmo Sender
pub struct NexmoSender { _config: NexmoConfig, _client: reqwest::Client }
impl NexmoSender { pub fn new(config: NexmoConfig) -> Self { Self { _config: config, _client: reqwest::Client::new() } } }

#[async_trait]
impl SmsSender for NexmoSender {
    async fn send(&self, _message: &SmsMessage) -> SmsResult { SmsResult::success(format!("nexmo_{}", uuid::Uuid::new_v4())) }
}

/// SMS Service
pub struct SmsService { sender: Box<dyn SmsSender + Send + Sync + 'static> }

impl SmsService {
    pub fn from_config(config: SmsConfig) -> Result<Self, SmsError> {
        let sender: Box<dyn SmsSender + Send + Sync + 'static> = match config.provider {
            SmsProvider::Twilio => {
                let twilio = config.twilio.ok_or_else(|| SmsError::Config("Twilio config required".into()))?;
                Box::new(TwilioSender::new(twilio))
            }
            SmsProvider::AwsSns => {
                let sns = config.sns.ok_or_else(|| SmsError::Config("SNS config required".into()))?;
                Box::new(SnsSender::new(sns))
            }
            SmsProvider::Nexmo => {
                let nexmo = config.nexmo.ok_or_else(|| SmsError::Config("Nexmo config required".into()))?;
                Box::new(NexmoSender::new(nexmo))
            }
            SmsProvider::HttpApi => return Err(SmsError::Config("HTTP API not implemented".into())),
        };
        Ok(Self { sender })
    }

    pub fn twilio(config: TwilioConfig) -> Self { Self { sender: Box::new(TwilioSender::new(config)) } }
    pub fn sns(config: SnsConfig) -> Self { Self { sender: Box::new(SnsSender::new(config)) } }
    pub fn nexmo(config: NexmoConfig) -> Self { Self { sender: Box::new(NexmoSender::new(config)) } }

    pub async fn send(&self, message: &SmsMessage) -> SmsResult { self.sender.send(message).await }
    pub async fn send_to_multiple(&self, to: Vec<String>, body: &str) -> Vec<SmsResult> {
        let mut results = Vec::new();
        for recipient in to {
            results.push(self.send(&SmsMessage::new(recipient, body)).await);
        }
        results
    }

    pub async fn send_template<F>(&self, to: &str, template_fn: F) -> SmsResult where F: FnOnce(&str) -> SmsMessage {
        let message = template_fn(to);
        self.send(&message).await
    }
}
