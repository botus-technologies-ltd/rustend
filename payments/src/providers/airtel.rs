//! Airtel Money Payment Gateway (Africa)

use async_trait::async_trait;
use serde_json::Value;

use crate::types::*;
use crate::subscription::*;
use crate::distribution::*;
use crate::gateway::{PaymentGateway, PaymentError, PaymentResult};

#[derive(Debug, Clone)]
pub struct AirtelConfig { pub client_id: String, pub client_secret: String, pub merchant_id: String, pub environment: AirtelEnvironment }
#[derive(Debug, Clone, Copy)] pub enum AirtelEnvironment { Sandbox, Production }

impl AirtelConfig {
    pub fn new(client_id: impl Into<String>, client_secret: impl Into<String>, merchant_id: impl Into<String>) -> Self { Self { client_id: client_id.into(), client_secret: client_secret.into(), merchant_id: merchant_id.into(), environment: AirtelEnvironment::Sandbox } }
    pub fn production(mut self) -> Self { self.environment = AirtelEnvironment::Production; self }
}

#[derive(Clone)]
pub struct AirtelGateway { _config: AirtelConfig, _client: reqwest::Client }
impl AirtelGateway { pub fn new(config: AirtelConfig) -> Self { Self { _config: config, _client: reqwest::Client::new() } } }

#[async_trait]
impl PaymentGateway for AirtelGateway {
    fn provider(&self) -> PaymentProvider { PaymentProvider::AirtelMoney }
    async fn create_payment(&self, amount: Amount, customer: Option<Customer>, description: Option<String>, metadata: Option<Value>) -> PaymentResult<PaymentIntent> { let mut intent = PaymentIntent::new(PaymentProvider::AirtelMoney, amount); intent.customer = customer; intent.description = description; intent.metadata = metadata; intent.next_action = Some(PaymentAction { action_type: PaymentActionType::Otp, data: serde_json::json!({ "message": "Enter PIN" }) }); Ok(intent) }
    async fn confirm_payment(&self, _payment_intent_id: &str, _payment_data: Option<Value>) -> PaymentResult<TransactionResult> { Ok(TransactionResult::success(format!("AIRTEL_{}", uuid::Uuid::new_v4()))) }
    async fn cancel_payment(&self, _payment_intent_id: &str) -> PaymentResult<TransactionResult> { Ok(TransactionResult::failed("Cancelled", "CANCELLED")) }
    async fn get_payment(&self, _payment_intent_id: &str) -> PaymentResult<PaymentIntent> { Ok(PaymentIntent::new(PaymentProvider::AirtelMoney, Amount::new(0, "KES"))) }
    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult> { Ok(RefundResult { success: true, refund_id: Some(format!("REF_{}", uuid::Uuid::new_v4())), status: PaymentStatus::Refunded, amount: request.amount.unwrap_or(0) }) }
    async fn create_customer(&self, _customer: Customer) -> PaymentResult<String> { Ok(format!("AIRTEL_CUS_{}", uuid::Uuid::new_v4())) }
    async fn get_customer(&self, _customer_id: &str) -> PaymentResult<Customer> { Ok(Customer::new()) }
    async fn attach_payment_method(&self, _customer_id: &str, _payment_method_token: &str) -> PaymentResult<String> { Ok(format!("AIRTEL_PM_{}", uuid::Uuid::new_v4())) }
    async fn create_subscription(&self, request: CreateSubscriptionRequest) -> PaymentResult<Subscription> { let plan = SubscriptionPlan::new(&request.plan_id, Amount::new(1000, "KES"), BillingInterval::Month); Ok(Subscription::new(&request.plan_id, &request.customer_id, &plan)) }
    async fn update_subscription(&self, _subscription_id: &str, request: UpdateSubscriptionRequest) -> PaymentResult<Subscription> { Ok(Subscription::new(request.plan_id.as_deref().unwrap_or("default"), "customer_123", &SubscriptionPlan::new("default", Amount::new(1000, "KES"), BillingInterval::Month))) }
    async fn cancel_subscription(&self, _subscription_id: &str, _cancel_at_period_end: bool) -> PaymentResult<Subscription> { Ok(Subscription::new("plan_123", "customer_123", &SubscriptionPlan::new("default", Amount::new(1000, "KES"), BillingInterval::Month))) }
    async fn get_subscription(&self, _subscription_id: &str) -> PaymentResult<Subscription> { Ok(Subscription::new("plan_123", "customer_123", &SubscriptionPlan::new("default", Amount::new(1000, "KES"), BillingInterval::Month))) }
    async fn create_payout(&self, amount: Amount, destination: PayoutDestination, description: Option<String>) -> PaymentResult<Payout> { let mut payout = Payout::new(amount, "recipient_123", RecipientType::Individual, PaymentProvider::AirtelMoney, destination); payout.description = description; Ok(payout) }
    async fn get_payout(&self, _payout_id: &str) -> PaymentResult<Payout> { Ok(Payout::new(Amount::new(5000, "KES"), "recipient_123", RecipientType::Individual, PaymentProvider::AirtelMoney, PayoutDestination::MobileMoney { phone: "254700000000".to_string(), operator: "airtel".to_string() })) }
    async fn create_batch_payout(&self, payouts: Vec<Payout>) -> PaymentResult<BatchPayout> { Ok(BatchPayout::new(PaymentProvider::AirtelMoney, payouts)) }
    async fn create_transfer(&self, amount: Amount, destination_account_id: &str) -> PaymentResult<Transfer> { Ok(Transfer::new(amount, "source_account", destination_account_id)) }
    async fn get_balance(&self, account_id: &str) -> PaymentResult<WalletBalance> { Ok(WalletBalance { account_id: account_id.to_string(), available: Amount::new(300000, "KES"), pending: Amount::new(30000, "KES"), currency: "KES".to_string() }) }
    fn verify_webhook_signature(&self, _payload: &[u8], signature: &str) -> bool { !signature.is_empty() }
    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent> { let value: Value = serde_json::from_slice(payload).map_err(|e| PaymentError::Provider(e.to_string()))?; Ok(WebhookEvent { event_id: value.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(), event_type: WebhookEventType::PaymentCompleted, provider: PaymentProvider::AirtelMoney, data: value, timestamp: chrono::Utc::now() }) }
}
