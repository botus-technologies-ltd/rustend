//! Subscription Types
//! 
//! Defines subscription models for recurring payments.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{Amount, PaymentProvider};

/// Subscription plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPlan {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub amount: Amount,
    pub interval: BillingInterval,
    pub interval_count: u32,
    pub trial_days: Option<u32>,
    pub metadata: Option<serde_json::Value>,
}

impl SubscriptionPlan {
    pub fn new(name: impl Into<String>, amount: Amount, interval: BillingInterval) -> Self {
        Self {
            id: format!("plan_{}", Uuid::new_v4()),
            name: name.into(),
            description: None,
            amount,
            interval,
            interval_count: 1,
            trial_days: None,
            metadata: None,
        }
    }
}

/// Billing intervals
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BillingInterval {
    Day,
    Week,
    Month,
    Year,
}

/// Subscription status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubscriptionStatus {
    Active,
    Trialing,
    PastDue,
    Canceled,
    Unpaid,
    Paused,
}

/// Subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub id: String,
    pub plan_id: String,
    pub customer_id: String,
    pub status: SubscriptionStatus,
    pub current_period_start: DateTime<Utc>,
    pub current_period_end: DateTime<Utc>,
    pub cancel_at_period_end: bool,
    pub canceled_at: Option<DateTime<Utc>>,
    pub trial_start: Option<DateTime<Utc>>,
    pub trial_end: Option<DateTime<Utc>>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Subscription {
    pub fn new(plan_id: impl Into<String>, customer_id: impl Into<String>, plan: &SubscriptionPlan) -> Self {
        let now = Utc::now();
        let (trial_start, trial_end) = plan.trial_days
            .map(|days| {
                let end = now + chrono::Duration::days(days as i64);
                (Some(now), Some(end))
            })
            .unwrap_or((None, None));

        Self {
            id: format!("sub_{}", Uuid::new_v4()),
            plan_id: plan_id.into(),
            customer_id: customer_id.into(),
            status: if plan.trial_days.is_some() { SubscriptionStatus::Trialing } else { SubscriptionStatus::Active },
            current_period_start: now,
            current_period_end: now + plan.interval.to_duration() * plan.interval_count as i32,
            cancel_at_period_end: false,
            canceled_at: None,
            trial_start,
            trial_end,
            metadata: None,
            created_at: now,
            updated_at: now,
        }
    }

    pub fn is_active(&self) -> bool {
        matches!(self.status, SubscriptionStatus::Active | SubscriptionStatus::Trialing)
    }
}

impl BillingInterval {
    pub fn to_duration(&self) -> chrono::Duration {
        match self {
            BillingInterval::Day => chrono::Duration::days(1),
            BillingInterval::Week => chrono::Duration::days(7),
            BillingInterval::Month => chrono::Duration::days(30),
            BillingInterval::Year => chrono::Duration::days(365),
        }
    }
}

/// Subscription creation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateSubscriptionRequest {
    pub plan_id: String,
    pub customer_id: String,
    pub payment_method_id: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

/// Subscription update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateSubscriptionRequest {
    pub plan_id: Option<String>,
    pub cancel_at_period_end: Option<bool>,
    pub metadata: Option<serde_json::Value>,
}

/// Subscription invoice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    pub id: String,
    pub subscription_id: String,
    pub customer_id: String,
    pub amount: Amount,
    pub status: InvoiceStatus,
    pub paid_at: Option<DateTime<Utc>>,
    pub due_date: Option<DateTime<Utc>>,
    pub invoice_number: String,
    pub line_items: Vec<InvoiceLineItem>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl Invoice {
    pub fn new(subscription: &Subscription, plan: &SubscriptionPlan) -> Self {
        Self {
            id: format!("inv_{}", Uuid::new_v4()),
            subscription_id: subscription.id.clone(),
            customer_id: subscription.customer_id.clone(),
            amount: plan.amount.clone(),
            status: InvoiceStatus::Open,
            paid_at: None,
            due_date: Some(subscription.current_period_end),
            invoice_number: format!("INV-{}", Uuid::new_v4().to_string()[..8].to_uppercase()),
            line_items: vec![InvoiceLineItem {
                description: plan.name.clone(),
                quantity: 1,
                unit_amount: plan.amount.value,
                amount: plan.amount.value,
            }],
            metadata: None,
            created_at: Utc::now(),
        }
    }
}

/// Invoice status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    Open,
    Paid,
    Void,
    Uncollectible,
}

/// Invoice line item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    pub description: String,
    pub quantity: u32,
    pub unit_amount: i64,
    pub amount: i64,
}

/// Payment method for subscription
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptionPaymentMethod {
    pub id: String,
    pub customer_id: String,
    pub provider: PaymentProvider,
    pub is_default: bool,
    pub expires_at: Option<DateTime<Utc>>,
}
