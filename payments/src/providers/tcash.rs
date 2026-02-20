//! TCash Payment Gateway (Indonesia)

use async_trait::async_trait;
use serde_json::Value;

use crate::types::*;
use crate::subscription::*;
use crate::distribution::*;
use crate::gateway::{PaymentGateway, PaymentError, PaymentResult};

#[derive(Debug, Clone)]
pub struct TCashConfig { pub client_id: String, pub client_secret: String, pub merchant_id: String, pub environment: TCashEnvironment }
#[derive(Debug, Clone, Copy)] pub enum TCashEnvironment { Sandbox, Production }

impl TCashConfig {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, merchant_id: impl Into<String>) -> Self { Self { client_id: client_id.into(), client_secret: client_secret.into(), merchant_id: merchant_id.into(), environment: TCashEnvironment::Sandbox } }
    pub fn production(mut self) -> Self { self.environment = TCashEnvironment::Production; self }
}

#[derive(Clone)]
pub struct TCashGateway { _config: TCashConfig, _client: reqwest::Client }
impl TCashGateway { pub fn new(config: TCashConfig) -> Self { Self { _config: config, _client: reqwest::Client::new() } } }

#[async_trait]
impl PaymentGateway for TCashGateway {
    fn provider(&self) -> PaymentProvider { PaymentProvider::TCash }
    async fn create_payment(&self, amount: Amount, customer: Option<Customer>, description: Option<String>, metadata: Option<Value>) -> PaymentResult<PaymentIntent> { let mut intent = PaymentIntent::new(PaymentProvider::TCash, amount); intent.customer = customer; intent.description = description; intent.metadata = metadata; intent.next_action = Some(PaymentAction { action_type: PaymentActionType::Otp, data: serde_json::json!({ "message": "Enter PIN" }) }); Ok(intent) }
    async fn confirm_payment(&self, _payment_intent_id: &str, _payment_data: Option<Value>) -> PaymentResult<TransactionResult> { Ok(TransactionResult::success(format!("TCASH_{}", uuid::Uuid::new_v4()))) }
    async fn cancel_payment(&self, _payment_intent_id: &str) -> PaymentResult<TransactionResult> { Ok(TransactionResult::failed("Cancelled", "CANCELLED")) }
    async fn get_payment(&self, _payment_intent_id: &str) -> PaymentResult<PaymentIntent> { Ok(PaymentIntent::new(PaymentProvider::TCash, Amount::new(0, "IDR"))) }
    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult> { Ok(RefundResult { success: true, refund_id: Some(format!("REF_{}", uuid::Uuid::new_v4())), status: PaymentStatus::Refunded, amount: request.amount.unwrap_or(0) }) }
    async fn create_customer(&self, _customer: Customer) -> PaymentResult<String> { Ok(format!("TCASH_CUS_{}", uuid::Uuid::new_v4())) }
    async fn get_customer(&self, _customer_id: &str) -> PaymentResult<Customer> { Ok(Customer::new()) }
    async fn attach_payment_method(&self, _customer_id: &str, _payment_method_token: &str) -> PaymentResult<String> { Ok(format!("TCASH_PM_{}", uuid::Uuid::new_v4())) }
    async fn create_subscription(&self, request: CreateSubscriptionRequest) -> PaymentResult<Subscription> { let plan = SubscriptionPlan::new(&request.plan_id, Amount::new(150000, "IDR"), BillingInterval::Month); Ok(Subscription::new(&request.plan_id, &request.customer_id, &plan)) }
    async fn update_subscription(&self, _subscription_id: &str, request: UpdateSubscriptionRequest) -> PaymentResult<Subscription> { Ok(Subscription::new(request.plan_id.as_deref().unwrap_or("default"), "customer_123", &SubscriptionPlan::new("default", Amount::new(150000, "IDR"), BillingInterval::Month))) }
    async fn cancel_subscription(&self, _subscription_id: &str, _cancel_at_period_end: bool) -> PaymentResult<Subscription> { Ok(Subscription::new("plan_123", "customer_123", &SubscriptionPlan::new("default", Amount::new(150000, "IDR"), BillingInterval::Month))) }
    async fn get_subscription(&self, _subscription_id: &str) -> PaymentResult<Subscription> { Ok(Subscription::new("plan_123", "customer_123", &SubscriptionPlan::new("default", Amount::new(150000, "IDR"), BillingInterval::Month))) }
    async fn create_payout(&self, amount: Amount, destination: PayoutDestination, description: Option<String>) -> PaymentResult<Payout> { let mut payout = Payout::new(amount, "recipient_123", RecipientType::Individual, PaymentProvider::TCash, destination); payout.description = description; Ok(payout) }
    async fn get_payout(&self, _payout_id: &str) -> PaymentResult<Payout> { Ok(Payout::new(Amount::new(500000, "IDR"), "recipient_123", RecipientType::Individual, PaymentProvider::TCash, PayoutDestination::MobileMoney { phone: "081200000000".to_string(), operator: "telkomsel".to_string() })) }
    async fn create_batch_payout(&self, payouts: Vec<Payout>) -> PaymentResult<BatchPayout> { Ok(BatchPayout::new(PaymentProvider::TCash, payouts)) }
    async fn create_transfer(&self, amount: Amount, destination_account_id: &str) -> PaymentResult<Transfer> { Ok(Transfer::new(amount, "source_account", destination_account_id)) }
    async fn get_balance(&self, account_id: &str) -> PaymentResult<WalletBalance> { Ok(WalletBalance { account_id: account_id.to_string(), available: Amount::new(10000000, "IDR"), pending: Amount::new(1000000, "IDR"), currency: "IDR".to_string() }) }
    fn verify_webhook_signature(&self, _payload: &[u8], signature: &str) -> bool { !signature.is_empty() }
    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent> { let value: Value = serde_json::from_slice(payload).map_err(|e| PaymentError::Provider(e.to_string()))?; Ok(WebhookEvent { event_id: value.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(), event_type: WebhookEventType::PaymentCompleted, provider: PaymentProvider::TCash, data: value, timestamp: chrono::Utc::now() }) }
}
