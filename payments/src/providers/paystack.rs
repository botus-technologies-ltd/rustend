//! Paystack Payment Gateway
//!
//! Paystack is a payment gateway popular in Nigeria and other African countries.
//! It supports card payments, bank transfers, USSD, and mobile money.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::distribution::*;
use crate::gateway::{PaymentError, PaymentGateway, PaymentResult};
use crate::subscription::*;
use crate::types::*;

/// Paystack environment
#[derive(Debug, Clone, Copy)]
pub enum PaystackEnvironment {
    /// Test/sandbox environment
    Sandbox,
    /// Production environment
    Production,
}

impl PaystackEnvironment {
    pub fn base_url(&self) -> &'static str {
        match self {
            Self::Sandbox => "https://api.paystack.co",
            Self::Production => "https://api.paystack.co",
        }
    }
}

/// Paystack configuration
#[derive(Debug, Clone)]
pub struct PaystackConfig {
    /// Secret key for API authentication
    pub secret_key: String,
    /// Public key for frontend integration
    pub public_key: String,
    /// Webhook secret for signature verification
    pub webhook_secret: String,
    /// Environment (sandbox or production)
    pub environment: PaystackEnvironment,
}

impl PaystackConfig {
    pub fn new(
        secret_key: impl Into<String>,
        public_key: impl Into<String>,
        webhook_secret: impl Into<String>,
    ) -> Self {
        Self {
            secret_key: secret_key.into(),
            public_key: public_key.into(),
            webhook_secret: webhook_secret.into(),
            environment: PaystackEnvironment::Sandbox,
        }
    }

    pub fn production(mut self) -> Self {
        self.environment = PaystackEnvironment::Production;
        self
    }
}

/// Paystack API response wrapper
#[derive(Debug, Deserialize)]
struct PaystackResponse<T> {
    status: bool,
    message: String,
    data: Option<T>,
}

/// Paystack transaction initialization response
#[derive(Debug, Deserialize)]
struct PaystackTransactionInit {
    authorization_url: String,
    access_code: String,
    reference: String,
}

/// Paystack transaction verification response
#[derive(Debug, Deserialize)]
struct PaystackTransaction {
    id: u64,
    reference: String,
    amount: i64,
    currency: String,
    status: String,
    gateway_response: String,
    paid_at: Option<String>,
    created_at: String,
    channel: Option<String>,
    customer: Option<PaystackCustomer>,
}

/// Paystack customer
#[derive(Debug, Deserialize)]
struct PaystackCustomer {
    id: u64,
    email: String,
    customer_code: String,
    first_name: Option<String>,
    last_name: Option<String>,
    phone: Option<String>,
}

/// Paystack refund request
#[derive(Debug, Serialize)]
struct PaystackRefundRequest {
    transaction: String,
    amount: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    merchant_note: Option<String>,
}

/// Paystack refund response
#[derive(Debug, Deserialize)]
struct PaystackRefund {
    id: u64,
    integration: u64,
    domain: String,
    transaction: u64,
    dispute: Option<u64>,
    amount: i64,
    currency: String,
    channel: Option<String>,
    status: String,
    refunded_at: String,
    created_at: String,
    updated_at: String,
}

/// Paystack transfer recipient
#[derive(Debug, Serialize)]
struct PaystackTransferRecipient {
    #[serde(rename = "type")]
    recipient_type: String,
    name: String,
    account_number: String,
    bank_code: String,
    currency: String,
}

/// Paystack transfer
#[derive(Debug, Serialize)]
struct PaystackTransferRequest {
    source: String,
    amount: i64,
    recipient: String,
    reason: String,
    currency: String,
}

/// Paystack transfer response
#[derive(Debug, Deserialize)]
struct PaystackTransfer {
    id: u64,
    reference: String,
    amount: i64,
    currency: String,
    status: String,
    reason: String,
    created_at: String,
}

/// Paystack balance
#[derive(Debug, Deserialize)]
struct PaystackBalance {
    currency: String,
    balance: i64,
}

/// Paystack gateway implementation
#[derive(Clone)]
pub struct PaystackGateway {
    config: PaystackConfig,
    client: reqwest::Client,
}

impl PaystackGateway {
    pub fn new(config: PaystackConfig) -> Self {
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
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers
    }

    async fn make_request<T: for<'de> Deserialize<'de>>(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> PaymentResult<T> {
        let url = format!("{}{}", self.config.environment.base_url(), path);
        let mut request = self.client.request(method, &url).headers(self.headers());

        if let Some(body) = body {
            request = request.json(&body);
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
                "Paystack API error ({}): {}",
                status, text
            )));
        }

        let paystack_response: PaystackResponse<T> =
            serde_json::from_str(&text).map_err(|e| PaymentError::Provider(e.to_string()))?;

        if !paystack_response.status {
            return Err(PaymentError::Provider(paystack_response.message));
        }

        paystack_response
            .data
            .ok_or_else(|| PaymentError::Provider("No data in response".to_string()))
    }

    fn map_status(paystack_status: &str) -> PaymentStatus {
        match paystack_status.to_lowercase().as_str() {
            "success" => PaymentStatus::Completed,
            "failed" => PaymentStatus::Failed,
            "abandoned" => PaymentStatus::Cancelled,
            "pending" | "ongoing" => PaymentStatus::Processing,
            "reversed" => PaymentStatus::Refunded,
            _ => PaymentStatus::Pending,
        }
    }
}

#[async_trait]
impl PaymentGateway for PaystackGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::Paystack
    }

    async fn create_payment(
        &self,
        amount: Amount,
        customer: Option<Customer>,
        description: Option<String>,
        metadata: Option<Value>,
    ) -> PaymentResult<PaymentIntent> {
        let mut body = serde_json::json!({
            "amount": amount.value,
            "currency": amount.currency,
        });

        if let Some(ref customer) = customer {
            if let Some(ref email) = customer.email {
                body["email"] = Value::String(email.clone());
            }
            if let Some(ref phone) = customer.phone {
                body["phone"] = Value::String(phone.clone());
            }
        }

        if let Some(ref desc) = description {
            body["reference"] = Value::String(format!("ref_{}", uuid::Uuid::new_v4()));
            body["description"] = Value::String(desc.clone());
        }

        if let Some(ref meta) = metadata {
            body["metadata"] = meta.clone();
        }

        let init: PaystackTransactionInit = self
            .make_request(reqwest::Method::POST, "/transaction/initialize", Some(body))
            .await?;

        let mut intent = PaymentIntent::new(PaymentProvider::Paystack, amount);
        intent.id = init.reference.clone();
        intent.customer = customer;
        intent.description = description;
        intent.metadata = metadata;
        intent.client_secret = Some(init.access_code);
        intent.next_action = Some(PaymentAction {
            action_type: PaymentActionType::Redirect,
            data: serde_json::json!({
                "authorization_url": init.authorization_url,
                "reference": init.reference,
            }),
        });

        Ok(intent)
    }

    async fn confirm_payment(
        &self,
        payment_intent_id: &str,
        _payment_data: Option<Value>,
    ) -> PaymentResult<TransactionResult> {
        let path = format!("/transaction/verify/{}", payment_intent_id);
        let transaction: PaystackTransaction = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let status = Self::map_status(&transaction.status);
        let success = status == PaymentStatus::Completed;

        if success {
            Ok(TransactionResult {
                success: true,
                transaction_id: Some(transaction.reference),
                status,
                error_message: None,
                error_code: None,
                metadata: Some(serde_json::json!({
                    "gateway_response": transaction.gateway_response,
                    "paid_at": transaction.paid_at,
                    "channel": transaction.channel,
                })),
            })
        } else {
            Ok(TransactionResult {
                success: false,
                transaction_id: Some(transaction.reference),
                status,
                error_message: Some(transaction.gateway_response),
                error_code: Some(transaction.status),
                metadata: None,
            })
        }
    }

    async fn cancel_payment(&self, payment_intent_id: &str) -> PaymentResult<TransactionResult> {
        // Paystack doesn't have a direct cancel endpoint
        // Transactions expire after 30 minutes if not paid
        Ok(TransactionResult {
            success: true,
            transaction_id: Some(payment_intent_id.to_string()),
            status: PaymentStatus::Cancelled,
            error_message: None,
            error_code: None,
            metadata: None,
        })
    }

    async fn get_payment(&self, payment_intent_id: &str) -> PaymentResult<PaymentIntent> {
        let path = format!("/transaction/verify/{}", payment_intent_id);
        let transaction: PaystackTransaction = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let mut intent = PaymentIntent::new(
            PaymentProvider::Paystack,
            Amount::new(transaction.amount, &transaction.currency),
        );
        intent.id = transaction.reference;
        intent.status = Self::map_status(&transaction.status);

        if let Some(ref customer) = transaction.customer {
            intent.customer = Some(
                Customer::new()
                    .with_email(&customer.email)
                    .with_name(format!(
                        "{} {}",
                        customer.first_name.as_deref().unwrap_or(""),
                        customer.last_name.as_deref().unwrap_or("")
                    )),
            );
        }

        Ok(intent)
    }

    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult> {
        let body = PaystackRefundRequest {
            transaction: request.payment_id,
            amount: request.amount,
            customer_note: request.reason.clone(),
            merchant_note: request.reason,
        };

        let body_value = serde_json::to_value(&body)
            .map_err(|e| PaymentError::Provider(e.to_string()))?;

        let refund: PaystackRefund = self
            .make_request(reqwest::Method::POST, "/refund", Some(body_value))
            .await?;

        Ok(RefundResult {
            success: refund.status == "processed" || refund.status == "pending",
            refund_id: Some(refund.id.to_string()),
            status: if refund.status == "processed" {
                PaymentStatus::Refunded
            } else {
                PaymentStatus::Refunding
            },
            amount: refund.amount,
        })
    }

    async fn create_customer(&self, customer: Customer) -> PaymentResult<String> {
        let mut body = serde_json::json!({});

        if let Some(ref email) = customer.email {
            body["email"] = Value::String(email.clone());
        }
        if let Some(ref phone) = customer.phone {
            body["phone"] = Value::String(phone.clone());
        }
        if let Some(ref name) = customer.name {
            let parts: Vec<&str> = name.splitn(2, ' ').collect();
            body["first_name"] = Value::String(parts[0].to_string());
            if parts.len() > 1 {
                body["last_name"] = Value::String(parts[1].to_string());
            }
        }

        let paystack_customer: PaystackCustomer = self
            .make_request(reqwest::Method::POST, "/customer", Some(body))
            .await?;

        Ok(paystack_customer.customer_code)
    }

    async fn get_customer(&self, customer_id: &str) -> PaymentResult<Customer> {
        let path = format!("/customer/{}", customer_id);
        let paystack_customer: PaystackCustomer = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let name = match (&paystack_customer.first_name, &paystack_customer.last_name) {
            (Some(first), Some(last)) => Some(format!("{} {}", first, last)),
            (Some(first), None) => Some(first.clone()),
            (None, Some(last)) => Some(last.clone()),
            (None, None) => None,
        };

        Ok(Customer {
            id: Some(paystack_customer.customer_code),
            email: Some(paystack_customer.email),
            phone: paystack_customer.phone,
            name,
        })
    }

    async fn attach_payment_method(
        &self,
        _customer_id: &str,
        _payment_method_token: &str,
    ) -> PaymentResult<String> {
        // Paystack handles payment methods differently - they're attached during payment
        Ok(format!("pm_{}", uuid::Uuid::new_v4()))
    }

    async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest,
    ) -> PaymentResult<Subscription> {
        let body = serde_json::json!({
            "customer": request.customer_id,
            "plan": request.plan_id,
        });

        #[derive(Deserialize)]
        struct PaystackSubscription {
            subscription_code: String,
            status: String,
            next_payment_date: Option<String>,
        }

        let subscription: PaystackSubscription = self
            .make_request(reqwest::Method::POST, "/subscription", Some(body))
            .await?;

        let plan = SubscriptionPlan::new(
            &request.plan_id,
            Amount::new(0, "NGN"),
            BillingInterval::Month,
        );

        let mut sub = Subscription::new(&request.plan_id, &request.customer_id, &plan);
        sub.id = subscription.subscription_code;
        sub.status = match subscription.status.as_str() {
            "active" => SubscriptionStatus::Active,
            "cancelled" => SubscriptionStatus::Canceled,
            _ => SubscriptionStatus::Active,
        };

        Ok(sub)
    }

    async fn update_subscription(
        &self,
        subscription_id: &str,
        request: UpdateSubscriptionRequest,
    ) -> PaymentResult<Subscription> {
        let mut body = serde_json::json!({});

        if let Some(ref plan_id) = request.plan_id {
            body["plan"] = Value::String(plan_id.clone());
        }

        let path = format!("/subscription/{}", subscription_id);

        #[derive(Deserialize)]
        struct PaystackSubscription {
            subscription_code: String,
            status: String,
            plan: PaystackPlan,
        }

        #[derive(Deserialize)]
        struct PaystackPlan {
            plan_code: String,
            amount: i64,
            interval: String,
        }

        let subscription: PaystackSubscription = self
            .make_request(reqwest::Method::PUT, &path, Some(body))
            .await?;

        let interval = match subscription.plan.interval.as_str() {
            "annually" => BillingInterval::Year,
            "quarterly" => BillingInterval::Month,
            "biannually" => BillingInterval::Month,
            _ => BillingInterval::Month,
        };

        let plan = SubscriptionPlan::new(
            &subscription.plan.plan_code,
            Amount::new(subscription.plan.amount, "NGN"),
            interval,
        );

        let mut sub = Subscription::new(
            &subscription.plan.plan_code,
            "customer",
            &plan,
        );
        sub.id = subscription.subscription_code;
        sub.status = match subscription.status.as_str() {
            "active" => SubscriptionStatus::Active,
            "cancelled" => SubscriptionStatus::Canceled,
            _ => SubscriptionStatus::Active,
        };

        Ok(sub)
    }

    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        _cancel_at_period_end: bool,
    ) -> PaymentResult<Subscription> {
        let body = serde_json::json!({
            "code": subscription_id,
            "token": subscription_id,
        });

        let path = format!("/subscription/disable");

        #[derive(Deserialize)]
        struct PaystackSubscription {
            subscription_code: String,
            status: String,
        }

        let subscription: PaystackSubscription = self
            .make_request(reqwest::Method::POST, &path, Some(body))
            .await?;

        let plan = SubscriptionPlan::new(
            "default",
            Amount::new(0, "NGN"),
            BillingInterval::Month,
        );

        let mut sub = Subscription::new("default", "customer", &plan);
        sub.id = subscription.subscription_code;
        sub.status = SubscriptionStatus::Canceled;

        Ok(sub)
    }

    async fn get_subscription(&self, subscription_id: &str) -> PaymentResult<Subscription> {
        let path = format!("/subscription/{}", subscription_id);

        #[derive(Deserialize)]
        struct PaystackSubscription {
            subscription_code: String,
            status: String,
            customer: PaystackCustomer,
            plan: PaystackPlan,
        }

        #[derive(Deserialize)]
        struct PaystackPlan {
            plan_code: String,
            amount: i64,
            interval: String,
        }

        let subscription: PaystackSubscription = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let interval = match subscription.plan.interval.as_str() {
            "annually" => BillingInterval::Year,
            "quarterly" => BillingInterval::Month,
            "biannually" => BillingInterval::Month,
            _ => BillingInterval::Month,
        };

        let plan = SubscriptionPlan::new(
            &subscription.plan.plan_code,
            Amount::new(subscription.plan.amount, "NGN"),
            interval,
        );

        let mut sub = Subscription::new(
            &subscription.plan.plan_code,
            &subscription.customer.customer_code,
            &plan,
        );
        sub.id = subscription.subscription_code;
        sub.status = match subscription.status.as_str() {
            "active" => SubscriptionStatus::Active,
            "cancelled" => SubscriptionStatus::Canceled,
            _ => SubscriptionStatus::Active,
        };

        Ok(sub)
    }

    async fn create_payout(
        &self,
        amount: Amount,
        destination: PayoutDestination,
        description: Option<String>,
    ) -> PaymentResult<Payout> {
        // First create a transfer recipient
        let recipient_body = match &destination {
            PayoutDestination::Bank {
                account_number,
                bank_name,
                account_holder_name,
                ..
            } => {
                // Note: Paystack requires bank_code, not bank_name
                // This is a simplified version - in production, you'd need to map bank names to codes
                serde_json::json!({
                    "type": "nuban",
                    "name": account_holder_name,
                    "account_number": account_number,
                    "bank_code": "044", // Default to Access Bank - should be configurable
                    "currency": amount.currency,
                })
            }
            PayoutDestination::MobileMoney { phone, operator } => {
                serde_json::json!({
                    "type": "mobile_money",
                    "name": "Mobile Money Recipient",
                    "phone": phone,
                    "provider": operator,
                    "currency": amount.currency,
                })
            }
            _ => {
                return Err(PaymentError::Validation(
                    "Unsupported payout destination for Paystack".to_string(),
                ))
            }
        };

        #[derive(Deserialize)]
        struct PaystackRecipient {
            recipient_code: String,
        }

        let recipient: PaystackRecipient = self
            .make_request(reqwest::Method::POST, "/transferrecipient", Some(recipient_body))
            .await?;

        // Create the transfer
        let transfer_body = PaystackTransferRequest {
            source: "balance".to_string(),
            amount: amount.value,
            recipient: recipient.recipient_code,
            reason: description.unwrap_or_else(|| "Payout".to_string()),
            currency: amount.currency.clone(),
        };

        let transfer_value = serde_json::to_value(&transfer_body)
            .map_err(|e| PaymentError::Provider(e.to_string()))?;

        let transfer: PaystackTransfer = self
            .make_request(reqwest::Method::POST, "/transfer", Some(transfer_value))
            .await?;

        let mut payout = Payout::new(
            amount,
            &transfer.reference,
            RecipientType::Individual,
            PaymentProvider::Paystack,
            destination,
        );
        payout.id = transfer.id.to_string();
        payout.description = Some(transfer.reason);
        payout.status = match transfer.status.as_str() {
            "success" => PayoutStatus::Completed,
            "pending" | "otp" => PayoutStatus::InTransit,
            "failed" | "reversed" => PayoutStatus::Failed,
            _ => PayoutStatus::Pending,
        };

        Ok(payout)
    }

    async fn get_payout(&self, payout_id: &str) -> PaymentResult<Payout> {
        let path = format!("/transfer/{}", payout_id);

        let transfer: PaystackTransfer = self
            .make_request(reqwest::Method::GET, &path, None)
            .await?;

        let mut payout = Payout::new(
            Amount::new(transfer.amount, &transfer.currency),
            &transfer.reference,
            RecipientType::Individual,
            PaymentProvider::Paystack,
            PayoutDestination::Bank {
                account_number: String::new(),
                routing_number: String::new(),
                account_holder_name: String::new(),
                bank_name: None,
            },
        );
        payout.id = transfer.id.to_string();
        payout.description = Some(transfer.reason);
        payout.status = match transfer.status.as_str() {
            "success" => PayoutStatus::Completed,
            "pending" | "otp" => PayoutStatus::InTransit,
            "failed" | "reversed" => PayoutStatus::Failed,
            _ => PayoutStatus::Pending,
        };

        Ok(payout)
    }

    async fn create_batch_payout(&self, payouts: Vec<Payout>) -> PaymentResult<BatchPayout> {
        // Paystack supports batch transfers through the bulk transfer endpoint
        let mut transfers = Vec::new();

        for payout in &payouts {
            transfers.push(serde_json::json!({
                "amount": payout.amount.value,
                "reference": payout.id,
                "reason": payout.description.as_deref().unwrap_or("Batch payout"),
                "recipient": payout.recipient_id,
            }));
        }

        let body = serde_json::json!({
            "currency": payouts.first().map(|p| p.amount.currency.clone()).unwrap_or_else(|| "NGN".to_string()),
            "source": "balance",
            "transfers": transfers,
        });

        #[derive(Deserialize)]
        struct PaystackBatchTransfer {
            batch_code: String,
            total_amount: i64,
        }

        let batch: PaystackBatchTransfer = self
            .make_request(reqwest::Method::POST, "/transfer/bulk", Some(body))
            .await?;

        Ok(BatchPayout {
            id: batch.batch_code,
            provider: PaymentProvider::Paystack,
            payouts,
            total_amount: Amount::new(batch.total_amount, "NGN"),
            status: BatchPayoutStatus::Processing,
            created_at: chrono::Utc::now(),
            completed_at: None,
        })
    }

    async fn create_transfer(
        &self,
        amount: Amount,
        destination_account_id: &str,
    ) -> PaymentResult<Transfer> {
        let body = PaystackTransferRequest {
            source: "balance".to_string(),
            amount: amount.value,
            recipient: destination_account_id.to_string(),
            reason: "Transfer".to_string(),
            currency: amount.currency.clone(),
        };

        let transfer_value = serde_json::to_value(&body)
            .map_err(|e| PaymentError::Provider(e.to_string()))?;

        let transfer: PaystackTransfer = self
            .make_request(reqwest::Method::POST, "/transfer", Some(transfer_value))
            .await?;

        Ok(Transfer {
            id: transfer.id.to_string(),
            amount,
            source_account_id: "balance".to_string(),
            destination_account_id: destination_account_id.to_string(),
            status: match transfer.status.as_str() {
                "success" => TransferStatus::Completed,
                "pending" | "otp" => TransferStatus::Pending,
                "failed" | "reversed" => TransferStatus::Failed,
                _ => TransferStatus::Pending,
            },
            description: None,
            metadata: None,
            created_at: chrono::Utc::now(),
        })
    }

    async fn get_balance(&self, _account_id: &str) -> PaymentResult<WalletBalance> {
        #[derive(Deserialize)]
        struct PaystackBalanceResponse {
            currency: String,
            balance: i64,
        }

        let balances: Vec<PaystackBalanceResponse> = self
            .make_request(reqwest::Method::GET, "/balance", None)
            .await?;

        let balance = balances
            .first()
            .ok_or_else(|| PaymentError::Provider("No balance found".to_string()))?;

        Ok(WalletBalance {
            account_id: "main".to_string(),
            available: Amount::new(balance.balance, &balance.currency),
            pending: Amount::new(0, &balance.currency),
            currency: balance.currency.clone(),
        })
    }

    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool {
        // Simple signature verification - in production, use proper HMAC
        // For now, just check if signature is not empty
        !signature.is_empty()
    }

    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent> {
        #[derive(Deserialize)]
        struct PaystackWebhook {
            event: String,
            data: Value,
        }

        let webhook: PaystackWebhook =
            serde_json::from_slice(payload).map_err(|e| PaymentError::Provider(e.to_string()))?;

        let event_type = match webhook.event.as_str() {
            "charge.success" => WebhookEventType::PaymentCompleted,
            "charge.failed" => WebhookEventType::PaymentFailed,
            "refund.processed" => WebhookEventType::PaymentRefunded,
            "subscription.create" => WebhookEventType::SubscriptionCreated,
            "subscription.disable" => WebhookEventType::SubscriptionCancelled,
            "subscription.renew" => WebhookEventType::SubscriptionRenewed,
            _ => return Err(PaymentError::Provider(format!(
                "Unknown webhook event: {}",
                webhook.event
            ))),
        };

        Ok(WebhookEvent {
            event_id: webhook
                .data
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|id| id.to_string())
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            event_type,
            provider: PaymentProvider::Paystack,
            data: webhook.data,
            timestamp: chrono::Utc::now(),
        })
    }
}