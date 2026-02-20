//! Payment Configuration
//! 
//! Provides unified configuration for plug-and-play payment providers.

use std::sync::Arc;

use crate::types::PaymentProvider;
use crate::gateway::PaymentGateway;
use crate::providers::{VisaGateway, VisaConfig, PayPalGateway, PayPalConfig, MpesaGateway, MpesaConfig, AirtelGateway, AirtelConfig, TCashGateway, TCashConfig};

/// Payment configuration for a single provider
#[derive(Debug, Clone)]
pub enum ProviderConfig {
    Visa(VisaConfig),
    PayPal(PayPalConfig),
    Mpesa(MpesaConfig),
    AirtelMoney(AirtelConfig),
    TCash(TCashConfig),
}

impl ProviderConfig {
    pub fn visa(api_key: impl Into<String>, webhook_secret: impl Into<String>) -> Self {
        Self::Visa(VisaConfig::new(api_key, webhook_secret))
    }

    pub fn paypal(client_id: impl Into<String>, client_secret: impl Into<String>, webhook_id: impl Into<String>) -> Self {
        Self::PayPal(PayPalConfig::new(client_id, client_secret, webhook_id))
    }

    pub fn mpesa(consumer_key: impl Into<String>, consumer_secret: impl Into<String>, short_code: impl Into<String>, initiator_name: impl Into<String>, security_credential: impl Into<String>) -> Self {
        Self::Mpesa(MpesaConfig::new(consumer_key, consumer_secret, short_code, initiator_name, security_credential))
    }

    pub fn airtel(client_id: impl Into<String>, client_secret: impl Into<String>, merchant_id: impl Into<String>) -> Self {
        Self::AirtelMoney(AirtelConfig::new(client_id, client_secret, merchant_id))
    }

    pub fn tcash(client_id: impl Into<String>, client_secret: impl Into<String>, merchant_id: impl Into<String>) -> Self {
        Self::TCash(TCashConfig::new(client_id, client_secret, merchant_id))
    }

    pub fn build_gateway(self) -> Arc<dyn PaymentGateway> {
        match self {
            Self::Visa(config) => {
                let gateway = VisaGateway::new(config);
                Arc::new(gateway) as Arc<dyn PaymentGateway>
            }
            Self::PayPal(config) => {
                let gateway = PayPalGateway::new(config);
                Arc::new(gateway) as Arc<dyn PaymentGateway>
            }
            Self::Mpesa(config) => {
                let gateway = MpesaGateway::new(config);
                Arc::new(gateway) as Arc<dyn PaymentGateway>
            }
            Self::AirtelMoney(config) => {
                let gateway = AirtelGateway::new(config);
                Arc::new(gateway) as Arc<dyn PaymentGateway>
            }
            Self::TCash(config) => {
                let gateway = TCashGateway::new(config);
                Arc::new(gateway) as Arc<dyn PaymentGateway>
            }
        }
    }
}

/// Main payment configuration
#[derive(Debug, Clone)]
pub struct PaymentConfig {
    pub default_provider: PaymentProvider,
    pub providers: Vec<(PaymentProvider, ProviderConfig)>,
    pub test_mode: bool,
}

impl PaymentConfig {
    pub fn new() -> Self {
        Self { default_provider: PaymentProvider::Visa, providers: Vec::new(), test_mode: true }
    }

    pub fn with_default(mut self, provider: PaymentProvider) -> Self {
        self.default_provider = provider;
        self
    }

    pub fn add_provider(mut self, config: ProviderConfig) -> Self {
        let provider = match &config {
            ProviderConfig::Visa(_) => PaymentProvider::Visa,
            ProviderConfig::PayPal(_) => PaymentProvider::PayPal,
            ProviderConfig::Mpesa(_) => PaymentProvider::Mpesa,
            ProviderConfig::AirtelMoney(_) => PaymentProvider::AirtelMoney,
            ProviderConfig::TCash(_) => PaymentProvider::TCash,
        };
        self.providers.push((provider, config));
        self
    }

    pub fn test_mode(mut self, enabled: bool) -> Self {
        self.test_mode = enabled;
        self
    }

    pub fn build(&self) -> Arc<dyn PaymentGateway> {
        for (provider, config) in &self.providers {
            if *provider == self.default_provider {
                return config.clone().build_gateway();
            }
        }
        if let Some((_, config)) = self.providers.first() {
            return config.clone().build_gateway();
        }
        ProviderConfig::visa("default", "default").build_gateway()
    }

    pub fn get_gateway(&self, provider: PaymentProvider) -> Option<Arc<dyn PaymentGateway>> {
        for (p, config) in &self.providers {
            if *p == provider {
                return Some(config.clone().build_gateway());
            }
        }
        None
    }
}

impl Default for PaymentConfig {
    fn default() -> Self { Self::new() }
}

impl PaymentConfig {
    pub fn from_env() -> Self {
        let mut config = Self::new();
        if let (Ok(key), Ok(secret)) = (std::env::var("VISA_API_KEY"), std::env::var("VISA_WEBHOOK_SECRET")) {
            config = config.add_provider(ProviderConfig::visa(key, secret));
        }
        if let (Ok(id), Ok(secret), Ok(webhook)) = (
            std::env::var("PAYPAL_CLIENT_ID"),
            std::env::var("PAYPAL_CLIENT_SECRET"),
            std::env::var("PAYPAL_WEBHOOK_ID")
        ) {
            config = config.add_provider(ProviderConfig::paypal(id, secret, webhook));
        }
        if let (Ok(key), Ok(secret), Ok(code), Ok(initiator), Ok(cred)) = (
            std::env::var("M_PESA_CONSUMER_KEY"),
            std::env::var("M_PESA_CONSUMER_SECRET"),
            std::env::var("M_PESA_SHORT_CODE"),
            std::env::var("M_PESA_INITIATOR_NAME"),
            std::env::var("M_PESA_SECURITY_CREDENTIAL")
        ) {
            config = config.add_provider(ProviderConfig::mpesa(key, secret, code, initiator, cred));
        }
        config
    }
}
