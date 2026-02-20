//! Unified Payment Gateway Trait

use async_trait::async_trait;
use serde_json::Value;

use crate::types::*;
use crate::subscription::*;
use crate::distribution::*;

/// Errors
#[derive(Debug, thiserror::Error)]
pub enum PaymentError {
    #[error("Provider error: {0}")]
    Provider(String),
    #[error("Validation error: {0}")]
    Validation(String),
    #[error("Authentication error")]
    Authentication,
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Insufficient funds")]
    InsufficientFunds,
    #[error("Payment declined: {0}")]
    Declined(String),
    #[error("Network error: {0}")]
    Network(String),
    #[error("Configuration error: {0}")]
    Config(String),
}

pub type PaymentResult<T> = Result<T, PaymentError>;

/// Unified Payment Gateway Trait
#[async_trait]
pub trait PaymentGateway: Send + Sync {
    fn provider(&self) -> PaymentProvider;

    async fn create_payment(&self, amount: Amount, customer: Option<Customer>, description: Option<String>, metadata: Option<Value>) -> PaymentResult<PaymentIntent>;

    async fn confirm_payment(&self, payment_intent_id: &str, payment_data: Option<Value>) -> PaymentResult<TransactionResult>;

    async fn cancel_payment(&self, payment_intent_id: &str) -> PaymentResult<TransactionResult>;

    async fn get_payment(&self, payment_intent_id: &str) -> PaymentResult<PaymentIntent>;

    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult>;

    // Subscription
    async fn create_customer(&self, customer: Customer) -> PaymentResult<String>;
    async fn get_customer(&self, customer_id: &str) -> PaymentResult<Customer>;
    async fn attach_payment_method(&self, customer_id: &str, payment_method_token: &str) -> PaymentResult<String>;
    async fn create_subscription(&self, request: CreateSubscriptionRequest) -> PaymentResult<Subscription>;
    async fn update_subscription(&self, subscription_id: &str, request: UpdateSubscriptionRequest) -> PaymentResult<Subscription>;
    async fn cancel_subscription(&self, subscription_id: &str, cancel_at_period_end: bool) -> PaymentResult<Subscription>;
    async fn get_subscription(&self, subscription_id: &str) -> PaymentResult<Subscription>;

    // Payout
    async fn create_payout(&self, amount: Amount, destination: PayoutDestination, description: Option<String>) -> PaymentResult<Payout>;
    async fn get_payout(&self, payout_id: &str) -> PaymentResult<Payout>;
    async fn create_batch_payout(&self, payouts: Vec<Payout>) -> PaymentResult<BatchPayout>;
    async fn create_transfer(&self, amount: Amount, destination_account_id: &str) -> PaymentResult<Transfer>;
    async fn get_balance(&self, account_id: &str) -> PaymentResult<WalletBalance>;

    // Webhook
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool;
    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent>;
}
