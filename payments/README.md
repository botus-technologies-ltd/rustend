# Payments Crate

A unified payment gateway system supporting multiple payment providers with plug-and-play configuration.

## Supported Providers

- **Visa** - Card payments (Visa, Mastercard, etc.)
- **PayPal** - PayPal payments
- **M-Pesa** - Mobile money (Kenya)
- **Airtel Money** - Mobile money (Africa)
- **TCash** - Mobile money (Indonesia)

## Architecture

### Core Modules

```
payments/
├── lib.rs           # Main library entry point
├── types.rs         # Core payment types
├── subscription.rs  # Subscription models
├── distribution.rs  # Payout/distribution models
├── gateway.rs       # Unified PaymentGateway trait
├── config.rs        # Plug-and-play configuration
└── providers/       # Provider implementations
    ├── visa.rs
    ├── paypal.rs
    ├── mpesa.rs
    ├── airtel.rs
    └── tcash.rs
```

---

## Module Documentation

### 1. Types Module (`types.rs`)

Core payment types used across all providers.

#### Key Types:

| Type | Description |
|------|-------------|
| `PaymentProvider` | Enum: Visa, PayPal, Mpesa, AirtelMoney, TCash |
| `PaymentStatus` | Enum: Pending, Processing, Completed, Failed, Cancelled, Refunding, Refunded, RequiresAction |
| `Amount` | struct with `value` (i64) and `currency` (String) |
| `Customer` | struct with optional id, email, phone, name |
| `PaymentIntent` | Main payment request object |
| `PaymentMethod` | Payment method details (card, mobile money, etc.) |
| `TransactionResult` | Result returned after payment confirmation |
| `RefundRequest` | Refund request with payment_id, amount, reason |
| `RefundResult` | Refund operation result |
| `WebhookEvent` | Webhook event from payment provider |

#### Helper Functions:

```rust
// Create amount in specific currency
Amount::new(1000, "USD")    // 1000 cents = $10.00
Amount::usd(999)            // helper for USD
Amount::kes(100)            // helper for KES

// Create customer
Customer::new()
    .with_email("user@example.com")
    .with_phone("+254700000000")
    .with_name("John Doe")
```

---

### 2. Subscription Module (`subscription.rs`)

Handles recurring payments and billing.

#### Key Types:

| Type | Description |
|------|-------------|
| `BillingInterval` | Enum: Day, Week, Month, Year |
| `SubscriptionPlan` | Plan with amount, interval, trial_days |
| `Subscription` | Active subscription with period dates |
| `SubscriptionStatus` | Enum: Active, Trialing, PastDue, Canceled, Unpaid, Paused |
| `CreateSubscriptionRequest` | Request to create subscription |
| `UpdateSubscriptionRequest` | Request to update subscription |
| `Invoice` | Invoice for subscription |
| `InvoiceStatus` | Enum: Open, Paid, Void, Uncollectible |

#### Usage:

```rust
// Create a subscription plan
let plan = SubscriptionPlan::new(
    "Premium Plan",
    Amount::new(999, "USD"),
    BillingInterval::Month
).with_trial_days(14);

// Create subscription from plan
let subscription = Subscription::new(
    "plan_premium",
    "customer_123",
    &plan
);
```

---

### 3. Distribution Module (`distribution.rs`)

Handles payouts and fund distribution.

#### Key Types:

| Type | Description |
|------|-------------|
| `Payout` | Single payout to recipient |
| `PayoutStatus` | Enum: Pending, InTransit, Completed, Failed, Cancelled |
| `PayoutDestination` | Enum: Bank, MobileMoney, PayPal, Card |
| `RecipientType` | Enum: Individual, Business |
| `BatchPayout` | Multiple payouts in one request |
| `BatchPayoutStatus` | Enum: Pending, Processing, Completed, PartiallyCompleted, Failed |
| `Transfer` | Transfer between accounts |
| `TransferStatus` | Enum: Pending, Completed, Failed |
| `WalletBalance` | Account balance (available + pending) |
| `AccountStatement` | Transaction history |

#### Usage:

```rust
// Create single payout
let payout = Payout::new(
    Amount::new(5000, "KES"),
    "recipient_123",
    RecipientType::Individual,
    PaymentProvider::Mpesa,
    PayoutDestination::MobileMoney {
        phone: "254700000000".to_string(),
        operator: "safaricom".to_string()
    }
);

// Create bank payout
let bank_payout = Payout::new(
    Amount::new(10000, "USD"),
    "vendor_456",
    RecipientType::Business,
    PaymentProvider::Visa,
    PayoutDestination::Bank {
        account_number: "1234567890".to_string(),
        routing_number: "021000021".to_string(),
        account_holder_name: "Business Name".to_string(),
        bank_name: Some("Chase".to_string())
    }
);
```

---

### 4. Gateway Module (`gateway.rs`)

Defines the unified `PaymentGateway` trait that all providers must implement.

#### Trait Methods:

```rust
pub trait PaymentGateway: Send + Sync {
    // === Core Payment Operations ===
    
    /// Get the provider type
    fn provider(&self) -> PaymentProvider;
    
    /// Create a new payment intent
    async fn create_payment(
        &self,
        amount: Amount,
        customer: Option<Customer>,
        description: Option<String>,
        metadata: Option<Value>
    ) -> PaymentResult<PaymentIntent>;
    
    /// Confirm payment (for 3DS, OTP, etc.)
    async fn confirm_payment(
        &self,
        payment_intent_id: &str,
        payment_data: Option<Value>
    ) -> PaymentResult<TransactionResult>;
    
    /// Cancel pending payment
    async fn cancel_payment(&self, payment_intent_id: &str) -> PaymentResult<TransactionResult>;
    
    /// Get payment status
    async fn get_payment(&self, payment_intent_id: &str) -> PaymentResult<PaymentIntent>;
    
    /// Refund a payment
    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult>;
    
    // === Customer Management ===
    
    /// Create customer in provider
    async fn create_customer(&self, customer: Customer) -> PaymentResult<String>;
    
    /// Get customer details
    async fn get_customer(&self, customer_id: &str) -> PaymentResult<Customer>;
    
    /// Attach payment method to customer
    async fn attach_payment_method(
        &self,
        customer_id: &str,
        payment_method_token: &str
    ) -> PaymentResult<String>;
    
    // === Subscription Operations ===
    
    /// Create subscription
    async fn create_subscription(
        &self,
        request: CreateSubscriptionRequest
    ) -> PaymentResult<Subscription>;
    
    /// Update subscription
    async fn update_subscription(
        &self,
        subscription_id: &str,
        request: UpdateSubscriptionRequest
    ) -> PaymentResult<Subscription>;
    
    /// Cancel subscription
    async fn cancel_subscription(
        &self,
        subscription_id: &str,
        cancel_at_period_end: bool
    ) -> PaymentResult<Subscription>;
    
    /// Get subscription
    async fn get_subscription(&self, subscription_id: &str) -> PaymentResult<Subscription>;
    
    // === Payout/Distribution ===
    
    /// Create payout
    async fn create_payout(
        &self,
        amount: Amount,
        destination: PayoutDestination,
        description: Option<String>
    ) -> PaymentResult<Payout>;
    
    /// Get payout status
    async fn get_payout(&self, payout_id: &str) -> PaymentResult<Payout>;
    
    /// Create batch payout
    async fn create_batch_payout(&self, payouts: Vec<Payout>) -> PaymentResult<BatchPayout>;
    
    /// Transfer between accounts
    async fn create_transfer(
        &self,
        amount: Amount,
        destination_account_id: &str
    ) -> PaymentResult<Transfer>;
    
    /// Get wallet balance
    async fn get_balance(&self, account_id: &str) -> PaymentResult<WalletBalance>;
    
    // === Webhook Processing ===
    
    /// Verify webhook signature
    fn verify_webhook_signature(&self, payload: &[u8], signature: &str) -> bool;
    
    /// Parse webhook event
    fn parse_webhook_event(&self, payload: &[u8]) -> PaymentResult<WebhookEvent>;
}
```

#### Error Handling:

```rust
pub enum PaymentError {
    Provider(String),        // Provider-specific error
    Validation(String),     // Input validation error
    Authentication,         // Auth failed
    NotFound(String),       // Resource not found
    InsufficientFunds,      // Not enough funds
    Declined(String),      // Payment declined
    Network(String),       // Network error
    Config(String),        // Configuration error
}

pub type PaymentResult<T> = Result<T, PaymentError>;
```

---

### 5. Configuration Module (`config.rs`)

Provides plug-and-play configuration for payment providers.

#### Usage:

```rust
use payments::{PaymentConfig, ProviderConfig, PaymentProvider, Amount, PaymentGateway};

// Method 1: Programmatic configuration
let config = PaymentConfig::new()
    .add_provider(ProviderConfig::visa("sk_live_xxx", "whsec_xxx"))
    .add_provider(ProviderConfig::mpesa(
        "consumer_key",
        "consumer_secret", 
        "short_code",
        "initiator_name",
        "security_credential"
    ))
    .with_default(PaymentProvider::Visa)
    .test_mode(true);

let gateway = config.build();

// Method 2: From environment variables
// VISA_API_KEY, VISA_WEBHOOK_SECRET
// PAYPAL_CLIENT_ID, PAYPAL_CLIENT_SECRET, PAYPAL_WEBHOOK_ID
// M_PESA_CONSUMER_KEY, M_PESA_CONSUMER_SECRET, etc.
let config = PaymentConfig::from_env();

// Get specific provider
let mpesa = config.get_gateway(PaymentProvider::Mpesa);
```

---

## Provider Implementation Guide

### Implementing a New Provider

To add a new payment provider, create a new file in `providers/`:

```rust
// providers/my_provider.rs

use async_trait::async_trait;
use serde_json::Value;

use crate::types::*;
use crate::subscription::*;
use crate::distribution::*;
use crate::gateway::{PaymentGateway, PaymentError, PaymentResult};

// 1. Define Configuration
#[derive(Debug, Clone)]
pub struct MyProviderConfig {
    pub api_key: String,
    pub environment: MyProviderEnvironment,
}

#[derive(Debug, Clone, Copy)]
pub enum MyProviderEnvironment {
    Sandbox,
    Production,
}

impl MyProviderConfig {
    pub fn new(api_key: impl Into<String>) -> Self {
        Self { 
            api_key: api_key.into(), 
            environment: MyProviderEnvironment::Sandbox 
        }
    }
    
    pub fn production(mut self) -> Self {
        self.environment = MyProviderEnvironment::Production;
        self
    }
}

// 2. Define Gateway
#[derive(Clone)]
pub struct MyProviderGateway {
    config: MyProviderConfig,
    client: reqwest::Client,
}

impl MyProviderGateway {
    pub fn new(config: MyProviderConfig) -> Self {
        Self { 
            config, 
            client: reqwest::Client::new() 
        }
    }
    
    // Helper methods for API calls
    fn base_url(&self) -> &str {
        match self.config.environment {
            MyProviderEnvironment::Sandbox => "https://api.sandbox.provider.com",
            MyProviderEnvironment::Production => "https://api.provider.com",
        }
    }
}

// 3. Implement PaymentGateway trait
#[async_trait]
impl PaymentGateway for MyProviderGateway {
    fn provider(&self) -> PaymentProvider {
        PaymentProvider::MyProvider // Add to enum first!
    }
    
    async fn create_payment(
        &self, 
        amount: Amount, 
        customer: Option<Customer>, 
        description: Option<String>, 
        metadata: Option<Value>
    ) -> PaymentResult<PaymentIntent> {
        // TODO: Call provider API to create payment
        // Return PaymentIntent with client_secret for frontend
    }
    
    async fn confirm_payment(
        &self, 
        payment_intent_id: &str, 
        payment_data: Option<Value>
    ) -> PaymentResult<TransactionResult> {
        // TODO: Confirm payment with provider
        // Handle 3DS, OTP, etc.
    }
    
    async fn cancel_payment(&self, payment_intent_id: &str) -> PaymentResult<TransactionResult> {
        // TODO: Cancel pending payment
    }
    
    async fn get_payment(&self, payment_intent_id: &str) -> PaymentResult<PaymentIntent> {
        // TODO: Get payment status
    }
    
    async fn refund(&self, request: RefundRequest) -> PaymentResult<RefundResult> {
        // TODO: Process refund
    }
    
    // ... implement all other methods
}
```

### Adding New Provider to PaymentProvider Enum

In `types.rs`, add to the enum:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PaymentProvider {
    Visa,
    PayPal,
    Mpesa,
    AirtelMoney,
    TCash,
    MyProvider,  // Add here
}
```

---

## API Integration Examples

### One-Time Payment Flow

```rust
// 1. Create payment intent
let intent = gateway.create_payment(
    Amount::usd(2999),  // $29.99
    Some(Customer::new().with_email("user@example.com")),
    Some("Order #12345".to_string()),
    Some(serde_json::json!({ "order_id": "12345" }))
).await?;

// 2. Send client_secret to frontend
// Frontend uses this to collect payment details

// 3. Confirm payment (after user completes 3DS/OTP)
let result = gateway.confirm_payment(
    &intent.id,
    Some(serde_json::json!({ "payment_method": "card" }))
).await?;

if result.success {
    // Payment successful!
    println!("Transaction ID: {}", result.transaction_id);
}
```

### Subscription Flow

```rust
// 1. Create customer
let customer_id = gateway.create_customer(
    Customer::new()
        .with_email("user@example.com")
        .with_name("John Doe")
).await?;

// 2. Attach payment method (token from frontend)
let payment_method_id = gateway.attach_payment_method(
    &customer_id,
    "pm_token_from_frontend"
).await?;

// 3. Create subscription
let subscription = gateway.create_subscription(
    CreateSubscriptionRequest {
        plan_id: "plan_premium".to_string(),
        customer_id,
        payment_method_id: Some(payment_method_id),
        metadata: None
    }
).await?;
```

### Payout Flow

```rust
// Create payout to mobile money
let payout = gateway.create_payout(
    Amount::new(5000, "KES"),
    PayoutDestination::MobileMoney {
        phone: "254700000000".to_string(),
        operator: "safaricom".to_string()
    },
    Some("Freelance payment".to_string())
).await?;

// Check payout status
let status = gateway.get_payout(&payout.id).await?;
```

---

## Webhook Handling

```rust
async fn handle_webhook(
    gateway: &dyn PaymentGateway,
    payload: &[u8],
    signature: &str
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Verify signature
    if !gateway.verify_webhook_signature(payload, signature) {
        return Err("Invalid signature".into());
    }
    
    // 2. Parse event
    let event = gateway.parse_webhook_event(payload)?;
    
    // 3. Handle event
    match event.event_type {
        WebhookEventType::PaymentCompleted => {
            // Update order status, send confirmation email, etc.
        }
        WebhookEventType::PaymentFailed => {
            // Handle failed payment
        }
        WebhookEventType::SubscriptionCreated => {
            // Activate user access
        }
        WebhookEventType::SubscriptionCancelled => {
            // Revoke access
        }
        _ => {}
    }
    
    Ok(())
}
```

---

## Environment Variables

| Variable | Provider | Description |
|----------|----------|-------------|
| `VISA_API_KEY` | Visa | API key |
| `VISA_WEBHOOK_SECRET` | Visa | Webhook secret |
| `PAYPAL_CLIENT_ID` | PayPal | Client ID |
| `PAYPAL_CLIENT_SECRET` | PayPal | Client secret |
| `PAYPAL_WEBHOOK_ID` | PayPal | Webhook ID |
| `M_PESA_CONSUMER_KEY` | M-Pesa | Consumer key |
| `M_PESA_CONSUMER_SECRET` | M-Pesa | Consumer secret |
| `M_PESA_SHORT_CODE` | M-Pesa | Short code |
| `M_PESA_INITIATOR_NAME` | M-Pesa | Initiator name |
| `M_PESA_SECURITY_CREDENTIAL` | M-Pesa | Security credential |

---

## Testing

Run tests with:

```bash
cargo test -p payments
```

---

## License

MIT
