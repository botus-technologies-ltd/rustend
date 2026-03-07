//! Payments - Unified Payment Gateway System

pub mod config;
pub mod distribution;
pub mod gateway;
pub mod providers;
pub mod subscription;
pub mod types;

// Re-export key types
pub use config::{PaymentConfig, ProviderConfig};
pub use distribution::{Payout, PayoutDestination, WalletBalance};
pub use gateway::PaymentGateway;
pub use subscription::{BillingInterval, Subscription, SubscriptionPlan};
pub use types::{
    Amount, Customer, PaymentIntent, PaymentMethod, PaymentProvider, PaymentStatus, RefundRequest,
    RefundResult,
};
