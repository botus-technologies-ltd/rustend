//! Stripe Payment Gateway
//!
//! Stripe is a global payment gateway supporting cards, bank transfers, and various payment methods.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::distribution::*;
use crate::gateway::{PaymentError, PaymentGateway, PaymentResult};
use crate::subscription::*;
use crate::types::*;

/// Stripe environment
#[derive(Debug, Clone, Copy)]
pub enum StripeEnvironment {
    /// Test/sandbox environment
    Sandbox,
    /// Production environment
    Production,
}

impl StripeEnvironment {
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::Sandbox => "https://api.stripe.com/v1",
            Self::Production => "https://api.stripe.com/v1",
        }
    }
}

/// Stripe configuration
#[derive(Debug, Clone)]
pub struct StripeConfig {
    /// Secret key for API authentication
    pub secret_key: String,
    /// Publishable key for frontend integration
    pub publishable_key: String,
    /// Webhook secret for signature verification
    pub webhook_secret: String,
    /// Environment (sandbox or production)
    pub environment: StripeEnvironment,
}

impl StripeConfig {
    pub fn new(
        secret_key: impl Into<String>,
        publishable_key: impl Into<String>,
        webhook_secret: impl Into<String>,
    ) -> Self {
        Self {
            secret_key: secret_key.into(),
            publishable_key: publishable_key.into(),
            webhook_secret: webhook_secret.into(),
            environment: StripeEnvironment::Sandbox,
        }
    }

    pub fn production(mut self) -> Self {
        self.environment = StripeEnvironment::Production;
        self
    }
}

/// Stripe payment intent
#[derive(Debug, Deserialize)]
struct StripePaymentIntent {
    id: String,
    amount: i64,
    currency: String,
    status: String,
    client_secret: Option<String>,
    customer: Option<String>,
    description: Option<String>,
    metadata: Option<Value>,
    #[serde(rename = "latest_charge")]
    latest_charge: Option<String>,
}

/// Stripe customer
#[derive(Debug, Deserialize)]
struct StripeCustomer {
    id: String,
    email: Option<String>,
    name: Option<String>,
    phone: Option<String>,
    metadata: Option<Value>,
}

/// Stripe refund
#[derive(Debug, Deserialize)]
struct StripeRefund {
    id: String,
    amount: i64,
    currency: String,
    status: String,
    charge: Option<String>,
}

/// Stripe subscription
#[derive(Debug, Deserialize)]
struct StripeSubscription {
    id: String,
    status: String,
    customer: String,
    #[serde(rename = "current_period_start")]
    current_period_start: i64,
    #[serde(rename = "current_period_end")]
    current_period_end: i64,
    #[serde(rename = "cancel_at_period_end")]
    cancel_at_period_end: bool,
    items: StripeSubscriptionItems,
}

/// Stripe subscription items
#[derive(Debug, Deserialize)]
struct StripeSubscriptionItems {
    data: Vec<StripeSubscriptionItem>,
}

/// Stripe subscription item
#[derive(Debug, Deserialize)]
struct StripeSubscriptionItem {
    id: String,
    price: StripePrice,
}

/// Stripe price
#[derive(Debug, Deserialize)]
struct StripePrice {
    id: String,
    unit_amount: Option<i64>,
    currency: String,
    #[serde(rename = "recurring")]
    recurring: Option<StripeRecurring>,
}

/// Stripe recurring
#[derive(Debug, Deserialize)]
struct StripeRecurring {
    interval: String,
    #[serde(rename = "interval_count")]
    interval_count: u32,
}

/// Stripe transfer
#[derive(Debug, Deserialize)]
struct StripeTransfer {
    id: String,
    amount: i64,
    currency: String,
    destination: String,
}

/// Stripe balance
#[derive(Debug, Deserialize)]
struct StripeBalance {
    available: Vec<StripeBalanceAmount>,
    pending: Vec<StripeBalanceAmount>,
}

/// Stripe balance amount
#[derive(Debug, Deserialize)]
struct StripeBalanceAmount {
    amount: i64,
    currency: String,
}

/// Stripe payout
#[derive(Debug, Deserialize)]
struct StripePayout {
    id: String,
    amount: i64,
    currency: String,
    status: String,
    #[serde(rename = "arrival_date")]
    arrival_date: Option<i64>,
    description: Option<String>,
}

/// Stripe webhook event
#[derive(Debug, Deserialize)]
struct StripeWebhookEvent {
    id: String,
    #[serde(rename = "type")]
    event_type: String,
    data: StripeWebhookData,
    created: i64,
}

/// Stripe webhook data
#[derive(Debug, Deserialize)]
struct StripeWebhookData {
    object: Value,
}

/// Stripe gateway implementation
#[derive(Clone)]
pub struct StripeGateway {
    config: StripeConfig,
    client: reqwest::Client,
}

impl StripeGateway {
    pub fn new(config: StripeConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    fn headers(&self) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", self.config.secret_key))
                .expect("Invalid secret key"),
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/x-www-form-urlencoded"));
        headers
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<String>,
    ) -> PaymentResult<T> {
        let url = format!("{}{}", self.config.environment.base_url(), path);
        let mut request = self.client.request(method, &url).headers(self.headers());

        if let Some(body) = body {
            request = request.body(body);
        }

        let response = request
            .send()
            .await
            .map_err(|e| PaymentError::Network(e.to_string()))?;

        let status = response.status();
        let text = response
            .text()
            .await
            .map_err(|e| PaymentError::Network(e.to_string()))?;

        if !status.is_success() {
            return Err(PaymentError::Provider(format!(
                "Stripe API error ({}): {}",
                status, text
            )));
        }

        serde_json::from_str(&text).map_err(|e| PaymentError::Provider(e.to_string()))
    }

    fn map_status(stripe_status: &str) -> PaymentStatus {
        match stripe_status {
            "succeeded" | "paid" | "active" => PaymentStatus::Completed,
            "requires_payment_method" | "requires_confirmation" | "requires_action" => PaymentStatus::Pending,
            "processing" => PaymentStatus::Processing,
            "canceled" => PaymentStatus::Cancelled,
            "requires_capture" => PaymentStatus::Processing,
            _ => PaymentStatus::Pending,
        }
    }

    fn map_subscription_status(stripe_status: &str) -> SubscriptionStatus {
        match stripe_status {
            "active" | "trialing" => SubscriptionStatus::Active,
            "past_due" => SubscriptionStatus::PastDue,
            "canceled" | "unpaid" => SubscriptionStatus::Canceled,
            "paused" => SubscriptionStatus::Paused,
            _ => SubscriptionStatus::Active,
        }
    }

    fn map_payout_status(stripe_status: &str) -> PayoutStatus {
        match stripe_status {
            "paid" => PayoutStatus::Completed,
            "pending" => PayoutStatus::InTransit,
            "in_transit" => PayoutStatus::InTransit,
            "canceled" | "failed" => PayoutStatus::Failed,
            _ => PayoutStatus::Pending,
        }
    }
}

#[async_trait]
impl PaymentGateway for StripeGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Visa // Stripe is typically used for card payments
    }

    async fn create_payment(
        &self,
        amount: Amount,
        customer: Option<Customer>,
        description: Option<String>,
        metadata: Option<Value>,
    ) -> PaymentResult<PaymentIntent> {
        let mut params = vec![
            format!("amount={}", amount.value),
            format!("currency={}", amount.currency.to_lowercase()),
            "automatic_payment_methods[enabled]=true".to_string(),
        ];

        if let Some(ref customer) = customer {
            if let Some(ref email) = customer.email {
                params.push(format!("receipt_email={}", email));
            }
        }

        if let Some(ref desc) = description {
            params.push(format!("description={}", desc));
        }

        if let Some(ref meta) = metadata {
            if let Value::Object(map) = meta {
                for (key, value) in map {
                    params.push(format!("metadata[{}]={}", key, value));
                }
            }
        }

        let body = params.join("&");

        let intent: StripePaymentIntent = self
            .make_request(reqwest::Method::POST, "/payment_intents", Some(body))
            .await?;

        let mut payment_intent = PaymentIntent::new(PaymentProvider::Visa, amount);
        payment_intent.id = intent.id;
        payment_intent.status = Self::map_status(&intent.status);
        payment_intent.client_secret = intent.client_secret;
        payment_intent.customer = customer;
        payment_intent.description = description;
        payment_intent.metadata = metadata;

        Ok(payment_intent)
    }

    async fn confirm_payment(
        &self,
        payment_intent_id: &str,
        payment_data: Option<Value>,
    ) -> PaymentResult<TransactionResult> {
        let mut params = vec![];

        if let Some(data) = payment_data {
            if let Value::Object(map) = data {
                for (key, value) in map {
                    params.push(format!("payment_method_data[type]={}", key));
                    if let Value::String(s) = value {
                        params.push(format!("payment_method_data[card][token]={}", s));
                    }
                }
            }
        }

        let body = if params.is_empty() {
            None
        } else {
            Some(params.join("&"))
        };

        let path = format!("/payment_intents/{}/confirm", payment_intent_id);
        let intent: StripePaymentIntent = self
            .make_request(reqwest::Method::POST, &path, body)
            .await?;

        let status = Self::map_status(&intent.status);
        let success = status == PaymentStatus::Completed;

        Ok(TransactionResult {
            success,
            transaction_id: Some(intent.id),
            status,
            error_message: if !success {
                Some("Payment requires additional action".to_string())
            } else {
                None
            },
            error_code: None,
            metadata: None,
        })
    }

    async fn cancel_payment(&self, payment_intent_id: &str) -> PaymentResult<TransactionResult> {
        let path = format!("/payment_intents/{}/cancel", payment_intent_id);
        let intent: StripePaymentIntent = self
            .make_request(reqwest::Method::POST, &path, None)
            .await?;

        Ok(TransactionResult {
            success: true,
            transaction_id: Some(intent.id),
            status: PaymentStatus::Cancelled,
            error_message: None,
            error_code: None,
            metadata: None,
        })
    }

    async fn get_payment(&self, payment_intent_id: &str) -> PaymentResult<PaymentIntent> {
        let path = format!("/payment_intents/{}", payment_intent_id);
        let intent: StripePaymentIntent = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let mut payment_intent = PaymentIntent::new(
            PaymentProvider::Visa,
            Amount::new(intent.amount, &intent.currency),
        );
        payment_intent.id = intent.id;
        payment_intent.status = Self::map_status(&intent.status);
        payment_intent.client_secret = intent.client_secret;
        payment_intent.description = intent.description;
        payment_intent.metadata = intent.metadata;

        Ok(payment_intent)
    }

    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult> {
        let mut params = vec![
            format!("charge={}", request.payment_id),
        ];

        if let Some(amount) = request.amount {
            params.push(format!("amount={}", amount));
        }

        if let Some(ref reason) = request.reason {
            params.push(format!("reason={}", reason));
        }

        let body = params.join("&");

        let refund: StripeRefund = self
            .make_request(reqwest::Method::POST, "/refunds", Some(body))
            .await?;

        Ok(RefundResult {
            success: refund.status == "succeeded",
            refund_id: Some(refund.id),
            status: if refund.status == "succeeded" {
                PaymentStatus::Refunded
            } else {
                PaymentStatus::Refunding
            },
            amount: refund.amount,
        })
    }

    async fn create_customer(&self, customer: Customer) -> PaymentResult<String> {
        let mut params = vec![];

        if let Some(ref email) = customer.email {
            params.push(format!("email={}", email));
        }

        if let Some(ref name) = customer.name {
            params.push(format!("name={}", name));
        }

        if let Some(ref phone) = customer.phone {
            params.push(format!("phone={}", phone));
        }

        let body = if params.is_empty() {
            None
        } else {
            Some(params.join("&"))
        };

        let stripe_customer: StripeCustomer = self
            .make_request(reqwest::Method::POST, "/customers", body)
            .await?;

        Ok(stripe_customer.id)
    }

    async fn get_customer(&self, customer_id: &str) -> PaymentResult<Customer> {
        let path = format!("/customers/{}", customer_id);
        let stripe_customer: StripeCustomer = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        Ok(Customer {
            id: Some(stripe_customer.id),
            email: stripe_customer.email,
            phone: stripe_customer.phone,
            name: stripe_customer.name,
        })
    }

    async fn attach_payment_method(
        &self,
        customer_id: &str,
        payment_method_token: &str,
    ) -> PaymentResult<String> {
        let params = format!("customer={}", customer_id);
        let path = format!("/payment_methods/{}/attach", payment_method_token);

        #[derive(Deserialize)]
        struct PaymentMethod {
            id: String,
        }

        let pm: PaymentMethod = self
            .make_request(reqwest::Method::POST, &path, Some(params))
            .await?;

        Ok(pm.id)
    }

    async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> PaymentResult<Subscription> {
        let params = vec![
            format!("customer={}", request.customer_id),
            format!("items[0][price]={}", request.plan_id),
        ];

        let body = params.join("&");

        let stripe_sub: StripeSubscription = self
            .make_request(reqwest::Method::POST, "/subscriptions", Some(body))
            .await?;

        let price = stripe_sub.items.data.first()
            .map(|item| &item.price)
            .ok_or_else(|| PaymentError::Provider("No price found".to_string()))?;

        let interval = match price.recurring.as_ref().map(|r| r.interval.as_str()) {
            Some("year") => BillingInterval::Year,
            Some("week") => BillingInterval::Week,
            Some("day") => BillingInterval::Day,
            _ => BillingInterval::Month,
        };

        let plan = SubscriptionPlan::new(
            &price.id,
            Amount::new(price.unit_amount.unwrap_or(0), &price.currency),
            interval,
        );

        let mut sub = Subscription::new(
            &request.plan_id,
            &request.customer_id,
            &plan,
        );
        sub.id = stripe_sub.id;
        sub.status = Self::map_subscription_status(&stripe_sub.status);
        sub.cancel_at_period_end = stripe_sub.cancel_at_period_end;

        Ok(sub)
    }

    async fn update_subscription(
        &self,
        subscription_id: &str,
        request: UpdateSubscriptionRequest,
    ) -> PaymentResult<Subscription> {
        let mut params = vec![];

        if let Some(ref plan_id) = request.plan_id {
            // To update a subscription, we need to get the current subscription first
            let path = format!("/subscriptions/{}", subscription_id);
            let current: StripeSubscription = self
                .make_request(reqwest::Method::GET, &path, None)
                .await?;

            if let Some(item) = current.items.data.first() {
                params.push(format!("items[0][id]={}", item.id));
                params.push(format!("items[0][price]={}", plan_id));
            }
        }

        if let Some(cancel_at_period_end) = request.cancel_at_period_end {
            params.push(format!("cancel_at_period_end={}", cancel_at_period_end));
        }

        let body = if params.is_empty() {
            None
        } else {
            Some(params.join("&"))
        };

        let path = format!("/subscriptions/{}", subscription_id);
        let stripe_sub: StripeSubscription = self
            .make_request(reqwest::Method::POST, &path, body)
            .await?;

        let price = stripe_sub.items.data.first()
            .map(|item| &item.price)
            .ok_or_else(|| PaymentError::Provider("No price found".to_string()))?;

        let interval = match price.recurring.as_ref().map(|r| r.interval.as_str()) {
            Some("year") => BillingInterval::Year,
            Some("week") => BillingInterval::Week,
            Some("day") => BillingInterval::Day,
            _ => BillingInterval::Month,
        };

        let plan = SubscriptionPlan::new(
            &price.id,
            Amount::new(price.unit_amount.unwrap_or(0), &price.currency),
            interval,
        );

        let mut sub = Subscription::new(
            &price.id,
            &stripe_sub.customer,
            &plan,
        );
        sub.id = stripe_sub.id;
        sub.status = Self::map_subscription_status(&stripe_sub.status);
        sub.cancel_at_period_end = stripe_sub.cancel_at_period_end;

        Ok(sub)
    }

    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        cancel_at_period_end: bool,
    ) -> PaymentResult<Subscription> {
        let path = if cancel_at_period_end {
            format!("/subscriptions/{}", subscription_id)
        } else {
            format!("/subscriptions/{}/cancel", subscription_id)
        };

        let body = if cancel_at_period_end {
            Some("cancel_at_period_end=true".to_string())
        } else {
            None
        };

        let stripe_sub: StripeSubscription = self
            .make_request(reqwest::Method::POST, &path, body)
            .await?;

        let price = stripe_sub.items.data.first()
            .map(|item| &item.price)
            .ok_or_else(|| PaymentError::Provider("No price found".to_string()))?;

        let interval = match price.recurring.as_ref().map(|r| r.interval.as_str()) {
            Some("year") => BillingInterval::Year,
            Some("week") => BillingInterval::Week,
            Some("day") => BillingInterval::Day,
            _ => BillingInterval::Month,
        };

        let plan = SubscriptionPlan::new(
            &price.id,
            Amount::new(price.unit_amount.unwrap_or(0), &price.currency),
            interval,
        );

        let mut sub = Subscription::new(
            &price.id,
            &stripe_sub.customer,
            &plan,
        );
        sub.id = stripe_sub.id;
        sub.status = SubscriptionStatus::Canceled;
        sub.cancel_at_period_end = stripe_sub.cancel_at_period_end;

        Ok(sub)
    }

    async fn get_subscription(&self, subscription_id: &str) -> PaymentResult<Subscription> {
        let path = format!("/subscriptions/{}", subscription_id);
        let stripe_sub: StripeSubscription = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let price = stripe_sub.items.data.first()
            .map(|item| &item.price)
            .ok_or_else(|| PaymentError::Provider("No price found".to_string()))?;

        let interval = match price.recurring.as_ref().map(|r| r.interval.as_str()) {
            Some("year") => BillingInterval::Year,
            Some("week") => BillingInterval::Week,
            Some("day") => BillingInterval::Day,
            _ => BillingInterval::Month,
        };

        let plan = SubscriptionPlan::new(
            &price.id,
            Amount::new(price.unit_amount.unwrap_or(0), &price.currency),
            interval,
        );

        let mut sub = Subscription::new(
            &price.id,
            &stripe_sub.customer,
            &plan,
        );
        sub.id = stripe_sub.id;
        sub.status = Self::map_subscription_status(&stripe_sub.status);
        sub.cancel_at_period_end = stripe_sub.cancel_at_period_end;

        Ok(sub)
    }

    async fn create_payout(
        &self,
        amount: Amount,
        destination: PayoutDestination,
        description: Option<String>,
    ) -> PaymentResult<Payout> {
        let mut params = vec![
            format!("amount={}", amount.value),
            format!("currency={}", amount.currency.to_lowercase()),
        ];

        match &destination {
            PayoutDestination::Bank { account_number, .. } => {
                // For bank payouts, we need to use a connected account
                // This is a simplified version - in production, you'd need proper Stripe Connect setup
                params.push(format!("destination={}", account_number));
            }
            PayoutDestination::Card { card_id } => {
                params.push(format!("destination={}", card_id));
            }
            _ => {
                return Err(PaymentError::Validation(
                    "Unsupported payout destination for Stripe".to_string(),
                ))
            }
        }

        if let Some(ref desc) = description {
            params.push(format!("description={}", desc));
        }

        let body = params.join("&");

        let stripe_payout: StripePayout = self
            .make_request(reqwest::Method::POST, "/payouts", Some(body))
            .await?;

        let mut payout = Payout::new(
            amount,
            &stripe_payout.id,
            RecipientType::Individual,
            PaymentProvider::Visa,
            destination,
        );
        payout.id = stripe_payout.id;
        payout.description = stripe_payout.description;
        payout.status = Self::map_payout_status(&stripe_payout.status);

        Ok(payout)
    }

    async fn get_payout(&self, payout_id: &str) -> PaymentResult<Payout> {
        let path = format!("/payouts/{}", payout_id);
        let stripe_payout: StripePayout = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let mut payout = Payout::new(
            Amount::new(stripe_payout.amount, &stripe_payout.currency),
            &stripe_payout.id,
            RecipientType::Individual,
            PaymentProvider::Visa,
            PayoutDestination::Bank {
                account_number: String::new(),
                routing_number: String::new(),
                account_holder_name: String::new(),
                bank_name: None,
            },
        );
        payout.id = stripe_payout.id;
        payout.description = stripe_payout.description;
        payout.status = Self::map_payout_status(&stripe_payout.status);

        Ok(payout)
    }

    async fn create_batch_payout(&self, payouts: Vec<Payout>) -> PaymentResult<BatchPayout> {
        // Stripe doesn't have a native batch payout API
        // We'll create individual payouts and return a batch result
        let mut completed_payouts = Vec::new();

        for payout in payouts {
            let result = self.create_payout(
                payout.amount,
                payout.destination.clone(),
                payout.description.clone(),
            ).await?;
            completed_payouts.push(result);
        }

        Ok(BatchPayout {
            id: format!("batch_{}", uuid::Uuid::new_v4()),
            provider: PaymentProvider::Visa,
            payouts: completed_payouts,
            total_amount: Amount::new(
                completed_payouts.iter().map(|p| p.amount.value).sum(),
                completed_payouts.first().map(|p| p.amount.currency.clone()).unwrap_or_else(|| "usd".to_string()),
            ),
            status: BatchPayoutStatus::Completed,
            created_at: chrono::Utc::now(),
            completed_at: Some(chrono::Utc::now()),
        })
    }

    async fn create_transfer(
        &self,
        amount: Amount,
        destination_account_id: &str,
    ) -> PaymentResult<Transfer> {
        let params = vec![
            format!("amount={}", amount.value),
            format!("currency={}", amount.currency.to_lowercase()),
            format!("destination={}", destination_account_id),
        ];

        let body = params.join("&");

        let stripe_transfer: StripeTransfer = self
            .make_request(reqwest::Method::POST, "/transfers", Some(body))
            .await?;

        Ok(Transfer {
            id: stripe_transfer.id,
            amount,
            source_account_id: "platform".to_string(),
            destination_account_id: stripe_transfer.destination,
            status: TransferStatus::Completed,
            description: None,
            metadata: None,
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_balance(&self, _account_id: &str) -> PaymentResult<WalletBalance> {
        let balance: StripeBalance = self
            .make_request(reqwest::Method::GET, "/balance", None)
            .await?;

        let available = balance.available.first()
            .map(|b| Amount::new(b.amount, &b.currency))
            .unwrap_or_else(|| Amount::new(0, "usd"));

        let pending = balance.pending.first()
            .map(|b| Amount::new(b.amount, &b.currency))
            .unwrap_or_else(|| Amount::new(0, "usd"));

        Ok(WalletBalance {
            account_id: "main".to_string(),
            available,
            pending,
            currency: available.currency.clone(),
        })
    }

    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Stripe webhook signature verification
        // Format: t=timestamp,v1=signature
        let parts: Vec<&str> = signature.split(',').collect();
        let timestamp = parts.iter()
            .find(|p| p.starts_with("t="))
            .map(|p| &p[2..]);
        let sig = parts.iter()
            .find(|p| p.starts_with("v1="))
            .map(|p| &p[3..]);

        if let (Some(_timestamp), Some(_sig)) = (timestamp, sig) {
            // In production, you should verify the signature using HMAC-SHA256
            // For now, just check if signature is not empty
            !signature.is_empty()
        } else {
            false
        }
    }

    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent> {
        let webhook: StripeWebhookEvent =
            serde_json::from_slice(payload).map_err(|e| PaymentError::Provider(e.to_string()))?;

        let event_type = match webhook.event_type.as_str() {
            "payment_intent.succeeded" => WebhookEventType::PaymentCompleted,
            "payment_intent.payment_failed" => WebhookEventType::PaymentFailed,
            "charge.refunded" => WebhookEventType::PaymentRefunded,
            "customer.subscription.created" => WebhookEventType::SubscriptionCreated,
            "customer.subscription.deleted" => WebhookEventType::SubscriptionCancelled,
            "customer.subscription.updated" => WebhookEventType::SubscriptionRenewed,
            _ => return Err(PaymentError::Provider(format!(
                "Unknown webhook event: {}",
                webhook.event_type
            ))),
        };

        Ok(WebhookEvent {
            event_id: webhook.id,
            event_type,
            provider: PaymentProvider::Visa,
            data: webhook.data.object,
            timestamp: chrono::DateTime::from_timestamp(webhook.created, 0)
                .unwrap_or_else(chrono::Utc::now),
        })
    }
}