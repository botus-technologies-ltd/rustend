//! Payment Distribution/Payout Types
//! 
//! Defines models for paying out to users (marketplaces, gig platforms, etc.)

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::types::{Amount, PaymentProvider};

/// Payout status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PayoutStatus {
    Pending,
    InTransit,
    Completed,
    Failed,
    Cancelled,
}

/// Payout request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Payout {
    pub id: String,
    pub amount: Amount,
    pub recipient_id: String,
    pub recipient_type: RecipientType,
    pub status: PayoutStatus,
    pub provider: PaymentProvider,
    pub destination: PayoutDestination,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub failure_reason: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl Payout {
    pub fn new(
        amount: Amount,
        recipient_id: impl Into<String>,
        recipient_type: RecipientType,
        provider: PaymentProvider,
        destination: PayoutDestination,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: format!("po_{}", Uuid::new_v4()),
            amount,
            recipient_id: recipient_id.into(),
            recipient_type,
            status: PayoutStatus::Pending,
            provider,
            destination,
            description: None,
            metadata: None,
            failure_reason: None,
            created_at: now,
            updated_at: now,
            completed_at: None,
        }
    }
}

/// Recipient types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecipientType {
    Individual,
    Business,
}

/// Payout destination
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PayoutDestination {
    /// Bank account (US)
    Bank {
        account_number: String,
        routing_number: String,
        account_holder_name: String,
        bank_name: Option<String>,
    },
    /// Mobile money
    MobileMoney {
        phone: String,
        operator: String,
    },
    /// PayPal
    PayPal {
        email: String,
    },
    /// Card
    Card {
        card_id: String,
    },
}

/// Batch payout for multiple recipients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchPayout {
    pub id: String,
    pub payouts: Vec<Payout>,
    pub total_amount: Amount,
    pub status: BatchPayoutStatus,
    pub provider: PaymentProvider,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl BatchPayout {
    pub fn new(provider: PaymentProvider, payouts: Vec<Payout>) -> Self {
        let total: i64 = payouts.iter().map(|p| p.amount.value).sum();
        Self {
            id: format!("bp_{}", Uuid::new_v4()),
            payouts,
            total_amount: Amount::new(total, "USD"),
            status: BatchPayoutStatus::Pending,
            provider,
            created_at: Utc::now(),
            completed_at: None,
        }
    }
}

/// Batch payout status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchPayoutStatus {
    Pending,
    Processing,
    Completed,
    PartiallyCompleted,
    Failed,
}

/// Transfer between accounts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transfer {
    pub id: String,
    pub amount: Amount,
    pub source_account_id: String,
    pub destination_account_id: String,
    pub status: TransferStatus,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

impl Transfer {
    pub fn new(
        amount: Amount,
        source_account_id: impl Into<String>,
        destination_account_id: impl Into<String>,
    ) -> Self {
        Self {
            id: format!("tr_{}", Uuid::new_v4()),
            amount,
            source_account_id: source_account_id.into(),
            destination_account_id: destination_account_id.into(),
            status: TransferStatus::Pending,
            description: None,
            metadata: None,
            created_at: Utc::now(),
        }
    }
}

/// Transfer status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransferStatus {
    Pending,
    Completed,
    Failed,
}

/// Wallet balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletBalance {
    pub account_id: String,
    pub available: Amount,
    pub pending: Amount,
    pub currency: String,
}

/// Account statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountStatement {
    pub account_id: String,
    pub transactions: Vec<Transaction>,
    pub balance: WalletBalance,
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
}

/// Transaction in account statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub transaction_type: TransactionType,
    pub amount: Amount,
    pub balance_after: i64,
    pub description: Option<String>,
    pub reference: Option<String>,
    pub timestamp: DateTime<Utc>,
}

/// Transaction types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Credit,
    Debit,
    Payout,
    Refund,
    Fee,
    Transfer,
}
