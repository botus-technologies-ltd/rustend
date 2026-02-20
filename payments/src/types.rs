//! Core Payment Types
//! 
//! Defines the unified payment types used across all payment providers.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Supported payment providers/gateways
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProvider {
    /// Visa/MasterCard through stripe-like API
    Visa,
    /// PayPal
    PayPal,
    /// M-Pesa (Kenya)
    Mpesa,
    /// Airtel Money (Africa)
    AirtelMoney,
    /// TCash (Indonesia)
    TCash,
}

impl Default for PaymentProvider {
    fn default() -> Self {
        Self::Visa
    }
}

impl std::fmt::Display for PaymentProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaymentProvider::Visa => write!(f, "visa"),
            PaymentProvider::PayPal => write!(f, "paypal"),
            PaymentProvider::Mpesa => write!(f, "mpesa"),
            PaymentProvider::AirtelMoney => write!(f, "airtel_money"),
            PaymentProvider::TCash => write!(f, "tcash"),
        }
    }
}

/// Payment transaction status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    /// Payment is pending
    Pending,
    /// Payment is being processed
    Processing,
    /// Payment completed successfully
    Completed,
    /// Payment failed
    Failed,
    /// Payment was cancelled
    Cancelled,
    /// Payment is being refunded
    Refunding,
    /// Payment was refunded
    Refunded,
    /// Payment requires user action
    RequiresAction,
}

impl Default for PaymentStatus {
    fn default() -> Self {
        Self::Pending
    }
}

/// Payment amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Amount {
    /// Amount in smallest currency unit (cents, kobo, etc.)
    pub value: i64,
    /// Currency code (ISO 4217)
    pub currency: String,
}

impl Amount {
    pub fn new(value: i64, currency: impl Into<String>) -> Self {
        Self { value, currency: currency.into() }
    }

    pub fn usd(cents: i64) -> Self {
        Self::new(cents, "USD")
    }

    pub fn kes(sents: i64) -> Self {
        Self::new(sents, "KES")
    }
}

/// Customer information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Customer {
    pub id: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub name: Option<String>,
}

impl Customer {
    pub fn new() -> Self {
        Self { id: None, email: None, phone: None, name: None }
    }

    pub fn with_email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    pub fn with_phone(mut self, phone: impl Into<String>) -> Self {
        self.phone = Some(phone.into());
        self
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

/// Payment method types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethodType {
    Card,
    BankTransfer,
    MobileMoney,
    Wallet,
    USSD,
}

impl Default for PaymentMethodType {
    fn default() -> Self {
        Self::Card
    }
}

/// Payment method details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentMethod {
    pub method_type: PaymentMethodType,
    pub provider: PaymentProvider,
    pub last4: Option<String>,
    pub expiry_month: Option<u32>,
    pub expiry_year: Option<u32>,
    pub brand: Option<String>,
    pub phone: Option<String>,
}

/// Payment intent - the main payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: String,
    pub provider: PaymentProvider,
    pub amount: Amount,
    pub status: PaymentStatus,
    pub customer: Option<Customer>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub client_secret: Option<String>,
    pub next_action: Option<PaymentAction>,
}

impl PaymentIntent {
    pub fn new(provider: PaymentProvider, amount: Amount) -> Self {
        let now = Utc::now();
        Self {
            id: format!("pi_{}", Uuid::new_v4()),
            provider,
            amount,
            status: PaymentStatus::Pending,
            customer: None,
            description: None,
            metadata: None,
            created_at: now,
            updated_at: now,
            expires_at: None,
            client_secret: None,
            next_action: None,
        }
    }
}

/// Action required from customer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentAction {
    pub action_type: PaymentActionType,
    pub data: serde_json::Value,
}

/// Types of payment actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentActionType {
    Redirect,
    Otp,
    Pin,
    Password,
    ThreeDSecure,
    PhoneCall,
}

/// Payment result from provider (returned after confirmation)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub success: bool,
    pub transaction_id: Option<String>,
    pub status: PaymentStatus,
    pub error_message: Option<String>,
    pub error_code: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

impl TransactionResult {
    pub fn success(transaction_id: impl Into<String>) -> Self {
        Self {
            success: true,
            transaction_id: Some(transaction_id.into()),
            status: PaymentStatus::Completed,
            error_message: None,
            error_code: None,
            metadata: None,
        }
    }

    pub fn failed(message: impl Into<String>, code: impl Into<String>) -> Self {
        Self {
            success: false,
            transaction_id: None,
            status: PaymentStatus::Failed,
            error_message: Some(message.into()),
            error_code: Some(code.into()),
            metadata: None,
        }
    }
}

/// Refund request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundRequest {
    pub payment_id: String,
    pub amount: Option<i64>, // None = full refund
    pub reason: Option<String>,
}

/// Refund result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefundResult {
    pub success: bool,
    pub refund_id: Option<String>,
    pub status: PaymentStatus,
    pub amount: i64,
}

/// Webhook event from payment provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookEvent {
    pub event_id: String,
    pub event_type: WebhookEventType,
    pub provider: PaymentProvider,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Webhook event types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebhookEventType {
    PaymentCompleted,
    PaymentFailed,
    PaymentRefunded,
    SubscriptionCreated,
    SubscriptionCancelled,
    SubscriptionRenewed,
}
