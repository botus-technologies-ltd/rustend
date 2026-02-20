# Utils - Reusable Utility Crate

A comprehensive utility crate for Rust backend applications providing logging, encryption, hashing, signatures, and standardized API responses.

## Modules

- [Response](#response) - Standardized API responses
- [Logger](#logger) - Crate-specific logging
- [Encryption](#encryption) - AES-256-GCM encryption
- [Hash](#hash) - Password hashing and OTP generation
- [Signature](#signature) - Request signing for secure communication
- [Email](#email) - Email sending (SMTP, SendGrid, AWS SES, Mailgun)
- [SMS](#sms) - SMS/Phone notifications (Twilio, AWS SNS, Nexmo)
- [WebSocket](#websocket) - Real-time communication (notifications, alerts, messaging)

---

## Response

Standardized API response types for consistent JSON responses across your application.

### Usage

```rust
use utils::response::{ApiResponse, ResponseMeta, ValidationError};

// Success response with data
let response = ApiResponse::success_data("User created", user);

// Success response without data
let response = ApiResponse::<()>::ok("Operation successful");

// Error response
let response = ApiResponse::<()>::error("Something went wrong", None);

// Validation error
let errors = vec![
    ValidationError { field: "email".into(), message: "Invalid email".into() }
];
let response = ApiResponse::validation_error(errors);

// Paginated response
let meta = ResponseMeta::new(1, 10, 50); // page, per_page, total
let response = ApiResponse::success_data("Users list", users).with_meta(meta);
```

### Response Format

```json
{
  "success": true,
  "message": "Operation successful",
  "data": { ... },
  "error": null,
  "meta": null
}
```

---

## Logger

Modular logging system where each crate logs to its own file in the `logs/` directory.

### Usage

Add to your crate's entry point (main.rs or lib.rs):

```rust
use utils::init;

fn main() {
    // Initialize logger - auto-detects crate name
    let _guard = init!();
    
    // Or with custom log level
    // let _guard = init!("debug");
    
    tracing::info!("Application started");
    tracing::warn!("Warning message");
    tracing::error!("Error occurred");
}
```

### Log Files

Each crate creates its own log file:
- `logs/app.log`
- `logs/auth.log`
- `logs/database.log`
- etc.

### Configuration

- Logs to both file and stdout
- File logs include: target, thread ID, file, line number
- Stdout logs include: target with ANSI colors
- Uses daily rotation

---

## Encryption

AES-256-GCM encryption for secure data transmission between backend and frontend.

### Features

- Industry-standard AES-256-GCM
- Authenticated encryption (confidentiality + integrity)
- Random nonce for each encryption
- Base64 encoded output (JSON safe)

### Usage

```rust
use utils::encryption::{AesGcmEncryption, generate_key};

// Create encryptor with 32-byte key
let key = generate_key();  // 32 random bytes
let encryptor = AesGcmEncryption::new(&key).unwrap();

// Encrypt data
let plaintext = "Sensitive data";
let encrypted = encryptor.encrypt(plaintext).unwrap();
// Returns: "base64string..."

// Decrypt data
let decrypted = encryptor.decrypt(&encrypted).unwrap();
assert_eq!(plaintext, decrypted);

// Generate keys in different formats
let key_hex = utils::encryption::generate_key_hex();
let key_base64 = utils::encryption::generate_key_base64();
```

### Security Notes

- Key must be exactly 32 bytes (256 bits)
- Each encryption generates a new random nonce
- Authentication tag verifies data integrity
- Store key securely (environment variables recommended)

---

## Hash

Secure password hashing and OTP generation.

### Features

- **Argon2** (recommended) - Winner of Password Hashing Competition
- **Bcrypt** - Widely supported
- OTP generation
- Random string generation

### Usage

```rust
use utils::hash::{Hash, Hasher};

// Hash a password (Argon2 recommended)
let hash = Hash::argon2("mypassword").unwrap();
let stored = hash.to_string();  // Store in DB

// Verify password
let parsed = Hash::from_string(&stored).unwrap();
assert!(parsed.verify("mypassword").unwrap());

// Using Hasher trait (one-liner)
let hash = "password".hash().unwrap();
assert!("password".verify(&hash).unwrap();

// Bcrypt (for compatibility)
let hash = Hash::bcrypt("password").unwrap();

// Custom parameters
let hash = Hash::argon2_custom("password", 65536, 3, 4);  // memory, iterations, parallelism

// Generate OTPs and random strings
let otp = utils::hash::generate_otp(6);  // "123456"
let random = utils::hash::generate_random(16);  // "Ab3dEfGhIjKlMnOp"
let hex = utils::hash::generate_hex(32);  // Hex string for API keys
```

### Hash Format

Argon2: `$argon2id$v=19$m=65536,t=3,p=4$...`
Bcrypt: `$2b$12$...`

---

## Signature

HMAC-SHA256 request signing to verify message integrity and detect tampering.

### Features

- Tamper detection - any message change causes verification to fail
- Timestamp-based replay protection
- Optional nonce for extra security
- Signed request builder

### Usage

```rust
use utils::signature::{Signer, Signature};

// Generate a shared secret key (store securely!)
let key = Signer::generate_key();

// Frontend: Sign a request
let message = "amount=100&to=account123";
let signature = Signer::sign(message, &key).unwrap();
// signature.timestamp is automatically set to current time

// Frontend sends: message + signature.signature + signature.timestamp

// Backend: Verify (rejects requests older than 5 minutes)
let is_valid = signature.verify(message, &key, 5).unwrap();
// If message was tampered: verify fails!

// Quick verify (without Signature struct)
let is_valid = Signer::quick_verify(
    message, 
    signature_str, 
    timestamp, 
    &key, 
    5  // max age in minutes
).unwrap();

// Signed request builder
let request = SignedRequest::new("POST", "/api/transfer")
    .with_query("lang=en")
    .with_body(r#"{"amount":100}"#)
    .sign(&key)
    .unwrap();

let json = request.to_json();
// Verify
assert!(request.verify(&key, 5).unwrap());

// URL signing
let url = utils::signature::create_signed_url(
    "/api/data",
    &[("id", "123"), ("action", "update")],
    &key
).unwrap();
```

### Security Flow

```
1. Backend generates key: Signer::generate_key()
2. Backend shares key with frontend (secure channel)
3. Frontend signs: signature = HMAC-SHA256(timestamp + message, key)
4. Frontend sends: message + signature + timestamp
5. Backend verifies: if HMAC matches → valid; else → tampered
6. Backend checks timestamp: if > max_age → rejected (replay protection)
```

### Signature Format

```json
{
  "signature": "base64encodedhmac...",
  "timestamp": 1699999999,
  "nonce": null
}
```

---

## Email

Generic email sending module supporting multiple providers (SMTP, SendGrid, AWS SES, Mailgun).

### Features

- Multiple provider support (SMTP, SendGrid, AWS SES, Mailgun)
- Fluent Email builder API
- Pre-built email templates
- Async trait-based EmailSender for custom implementations

### Usage

```rust
use utils::email::{EmailService, Email, templates};

// Using EmailService builder
let service = EmailService::sendgrid("api_key", "from@example.com");

// Build email manually
let email = Email::new("from@example.com", "to@example.com", "Subject")
    .html("<h1>Hello!</h1>")
    .text("Hello!")
    .cc(vec!["cc@example.com".to_string()])
    .bcc(vec!["bcc@example.com".to_string()]);

// Send
let result = service.send(&email).await;

// Use pre-built templates
let email = templates::welcome_email("user@example.com", "John");
service.send(&email).await;

let email = templates::password_reset("user@example.com", "reset_token_123");
service.send(&email).await;

let email = templates::verify_email("user@example.com", "verify_token_456");
service.send(&email).await;

let email = templates::order_confirmation("user@example.com", "ORD-123", "$99.99");
service.send(&email).await;
```

### Provider Setup

```rust
// SendGrid
let service = EmailService::sendgrid("SG.api_key", "from@example.com");

// AWS SES
let service = EmailService::ses("us-east-1", "from@example.com");

// Mailgun
let service = EmailService::mailgun("api_key", "your-domain.com");

// SMTP
let config = SmtpConfig::new("smtp.example.com", 587, "user", "pass");
let service = EmailService::smtp(config);
```

---

## SMS

Generic SMS/Phone notification module supporting multiple providers (Twilio, AWS SNS, Nexmo).

### Features

- Multiple provider support (Twilio, AWS SNS, Nexmo)
- SmsMessage builder API
- Pre-built SMS templates

### Usage

```rust
use utils::sms::{SmsService, SmsMessage, templates};

// Using SmsService builder
let service = SmsService::twilio(TwilioConfig::new("SID", "token", "+1234567890"));

// Build SMS manually
let message = SmsMessage::new("+0987654321", "Hello from the app!");

// Send
let result = service.send(&message).await;

// Use pre-built templates
let msg = templates::verification_code("+0987654321", "123456");
service.send(&msg).await;

let msg = templates::password_reset("+0987654321", "reset_code");
service.send(&msg).await;

let msg = templates::order_confirmation("+0987654321", "ORD-123");
service.send(&msg).await;
```

### Provider Setup

```rust
// Twilio
let config = TwilioConfig::new("ACCOUNT_SID", "AUTH_TOKEN", "+1234567890");
let service = SmsService::twilio(config);

// AWS SNS
let config = SnsConfig::new("us-east-1", "access_key", "secret_key")
    .with_sender_id("MyApp");
let service = SmsService::sns(config);

// Nexmo
let config = NexmoConfig::new("api_key", "api_secret", "MyApp");
let service = SmsService::nexmo(config);
```

---

## WebSocket

Generic WebSocket module for real-time communication (notifications, alerts, messaging).

### Features

- WebSocket message types (Text, JSON, Binary, Ping/Pong)
- Connection management (WsHub for tracking connections)
- Channel-based pub/sub messaging
- Message types: Notification, Alert, ChatMessage, LiveUpdate, Presence
- WsService for easy sending to users/channels

### Usage

```rust
use utils::websocket::{
    WsService, WsServerConfig, WsHub, WsMessage, 
    Notification, Alert, ChatMessage, LiveUpdate, Presence
};

// Create WebSocket service
let ws = WsService::new(WsServerConfig::default());

// Send notification to specific user
let notification = Notification::new("New Message", "You have a new message!")
    .with_icon("bell")
    .with_data(serde_json::json!({"type": "message", "chat_id": "123"}));
ws.notify_user("user_123", notification);

// Send alert to user
let alert = Alert::warning("Warning", "Your session is about to expire");
ws.alert_user("user_123", alert);

// Send chat message
let message = ChatMessage::new("sender_123", "Hello there!")
    .to_user("recipient_456")
    .from_name("John");
ws.send_chat("recipient_456", message);

// Broadcast live update to channel
let update = LiveUpdate::created("order", "ORD-123", serde_json::json!({"status": "shipped"}));
ws.broadcast_update("orders", update);

// Update user presence
let presence = Presence::online("user_123");
ws.update_presence("presence", presence);
```

### Direct Hub Usage

```rust
use utils::websocket::{WsHub, WsServerConfig, ConnectionInfo};

// Create hub
let hub = WsHub::new(WsServerConfig::default());

// Register connection
let conn_info = ConnectionInfo::new("conn_123")
    .with_user("user_456")
    .with_ip("192.168.1.1");
hub.register_connection(conn_info);

// Subscribe to channel
hub.subscribe("conn_123", "notifications");

// Broadcast to channel
let msg = WsMessage::json(r#"{"type":"alert","message":"Hello"}"#);
hub.broadcast_to_channel("notifications", msg).unwrap();

// Send to specific user
hub.send_to_user("user_456", WsMessage::text("Hello!"));
```

---

## Quick Start

Add to your Cargo.toml:

```toml
[dependencies]
utils = { path = "../utils" }
```

Import and use:

```rust
use utils::{init, response, encryption, hash, signature};

fn main() {
    // Logging
    let _guard = init!();
    
    // Encryption
    let encryptor = encryption::AesGcmEncryption::new(&encryption::generate_key()).unwrap();
    let encrypted = encryptor.encrypt("data").unwrap();
    
    // Hashing
    let hashed = hash::Hash::argon2("password").unwrap();
    
    // Signing
    let sig = signature::Signer::sign("message", &signature::Signer::generate_key()).unwrap();
    
    // Responses
    let resp = response::ApiResponse::ok("Success");
}
```

---

## Dependencies

All dependencies are managed in the workspace `Cargo.toml`. The utils crate uses:

- `serde` / `serde_json` - Serialization
- `tracing` / `tracing-subscriber` / `tracing-appender` - Logging
- `aes-gcm` - Encryption
- `argon2` / `bcrypt` - Password hashing
- `hmac` / `sha2` - Request signing
- `chrono` - Timestamps
- `base64` - Encoding
- `rand` - Random generation
- `urlencoding` - URL encoding
