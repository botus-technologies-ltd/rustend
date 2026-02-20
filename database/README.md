# Database Crate

Database abstraction layer supporting multiple database backends with flexible ID types.

## Supported Databases

- **MongoDB** - NoSQL document database
- **PostgreSQL** - Relational database
- **MySQL** - Relational database
- **SQLite** - Embedded database

## Architecture

```
database/
├── lib.rs           # Main entry point
├── init.rs          # Database initialization
├── mongo.rs         # MongoDB connection
├── postgres.rs      # PostgreSQL connection
├── mysql.rs         # MySQL connection
├── sqlite.rs        # SQLite connection
├── utils/           # Utilities
│   ├── mod.rs       # Module exports
│   ├── types.rs     # ID types
│   └── errors.rs    # Error types
└── README.md        # This file
```

---

## 1. Utilities Module (`utils/`)

### Database Types

| Type | Description |
|------|-------------|
| `DatabaseType` | Enum: MongoDB, PostgreSQL, MySQL, SQLite |
| `DatabaseId` | Flexible ID type (ObjectId, UUID, i64, String) |
| `ObjectId` | MongoDB ObjectId wrapper (12 bytes) |
| `DbId` | Type alias for common database ID |
| `IdError` | ID parsing/conversion errors |

### Usage

```rust
use database::utils::{DatabaseType, DatabaseId, ObjectId, generate_id, parse_id};

// Generate ID based on database type
let id = generate_id(DatabaseType::MongoDB);
// Result: DatabaseId::ObjectId(ObjectId(...))

let id = generate_id(DatabaseType::PostgreSQL);
// Result: DatabaseId::Uuid(Uuid(...))

// Parse ID from string
let id = parse_id(DatabaseType::MongoDB, "507f1f77bcf86cd799439011")?;
// Result: DatabaseId::ObjectId(ObjectId(...))

// Convert between formats
let id = DatabaseId::from_uuid("550e8400-e29b-41d4-a716-446655440000")?;
let string = id.to_string();
// Result: "550e8400-e29b-41d4-a716-446655440000"
```

### ObjectId Usage

```rust
use database::utils::ObjectId;

// Create from hex string
let oid = ObjectId::from_str("507f1f77bcf86cd799439011")?;

// Create new ObjectId (with timestamp)
let new_oid = ObjectId::new();

// Convert to string
let hex = oid.to_string();
// Result: "507f1f77bcf86cd799439011"

// Get bytes
let bytes = oid.as_bytes();
```

---

## 2. Initialization Module (`init.rs`)

### Key Types

| Type | Description |
|------|-------------|
| `DatabaseConfig` | Configuration for database connection |
| `Database` | Trait that all database connections implement |
| `init_database()` | Async function to initialize database |
| `default_connection_string()` | Get default connection string |

### Usage

```rust
use database::init::{DatabaseConfig, init_database, default_connection_string};
use database::utils::DatabaseType;

// Method 1: Manual configuration
let config = DatabaseConfig::new(
    DatabaseType::MongoDB,
    "mongodb://localhost:27017",
    "my_database"
)
.with_pool(max: 10, min: 2)
.with_timeout(30);

let db = init_database(config).await?;

// Method 2: Default connection strings
let mongo_uri = default_connection_string(DatabaseType::MongoDB);
// Result: "mongodb://localhost:27017"

let postgres_uri = default_connection_string(DatabaseType::PostgreSQL);
// Result: "postgres://postgres:password@localhost:5432/dbname"
```

### Database Trait

```rust
use database::init::Database;
use database::utils::{DatabaseType, DbResult};

pub trait Database: Send + Sync {
    fn db_type(&self) -> DatabaseType;
    fn db_name(&self) -> &str;
    fn ping(&self) -> DbResult<()>;
    fn close(&self) -> DbResult<()>;
}
```

---

## 3. Database-Specific Modules

### MongoDB (`mongo.rs`)

```rust
use database::{DatabaseConfig, init_database};
use database::utils::DatabaseType;

#[cfg(feature = "mongodb")]
{
    let config = DatabaseConfig::new(
        DatabaseType::MongoDB,
        "mongodb://username:password@host:27017",
        "database_name"
    );
    
    let db = init_database(config).await?;
    
    // Get MongoDB database
    let mongo_conn = db.as_any()
        .downcast_ref::<MongoConnection>();
    
    // Get collection
    let collection = mongo_conn.database()
        .collection::<Document>("users");
}
```

**Features needed in Cargo.toml:**
```toml
[dependencies]
database = { path = "../database", features = ["mongodb"] }
```

---

### PostgreSQL (`postgres.rs`)

```rust
use database::{DatabaseConfig, init_database};
use database::utils::DatabaseType;

#[cfg(feature = "postgres")]
{
    let config = DatabaseConfig::new(
        DatabaseType::PostgreSQL,
        "postgres://user:pass@localhost:5432/mydb",
        "mydb"
    );
    
    let db = init_database(config).await?;
}
```

**Features needed in Cargo.toml:**
```toml
[dependencies]
database = { path = "../database", features = ["postgres"] }
```

---

### MySQL (`mysql.rs`)

```rust
use database::{DatabaseConfig, init_database};
use database::utils::DatabaseType;

#[cfg(feature = "mysql")]
{
    let config = DatabaseConfig::new(
        DatabaseType::MySQL,
        "mysql://root:password@localhost:3306/mydb",
        "mydb"
    );
    
    let db = init_database(config).await?;
}
```

**Features needed in Cargo.toml:**
```toml
[dependencies]
database = { path = "../database", features = ["mysql"] }
```

---

### SQLite (`sqlite.rs`)

```rust
use database::{DatabaseConfig, init_database};
use database::utils::DatabaseType;

#[cfg(feature = "sqlite")]
{
    let config = DatabaseConfig::new(
        DatabaseType::SQLite,
        "./data.db",
        "mydb"
    );
    
    let db = init_database(config).await?;
}
```

**Features needed in Cargo.toml:**
```toml
[dependencies]
database = { path = "../database", features = ["sqlite"] }
```

---

## Full Example

### Cargo.toml

```toml
[package]
name = "my_app"
version = "0.1.0"

[dependencies]
database = { path = "../database", features = ["mongodb", "postgres"] }
tokio = { version = "1", features = ["full"] }
```

### Application Code

```rust
use database::init::{DatabaseConfig, init_database};
use database::utils::{DatabaseType, DatabaseId, ObjectId};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Choose database based on environment
    let db_type = std::env::var("DB_TYPE")
        .unwrap_or_else(|_| "mongodb".to_string());
    
    let config = match db_type.as_str() {
        "mongodb" => DatabaseConfig::new(
            DatabaseType::MongoDB,
            std::env::var("MONGODB_URI")
                .unwrap_or_else(|_| "mongodb://localhost:27017".to_string()),
            std::env::var("DATABASE_NAME")
                .unwrap_or_else(|_| "myapp".to_string()),
        ),
        "postgres" => DatabaseConfig::new(
            DatabaseType::PostgreSQL,
            std::env::var("POSTGRES_URI")
                .unwrap_or_else(|_| "postgres://localhost/mydb".to_string()),
            "mydb",
        ),
        _ => panic!("Unknown database type: {}", db_type),
    };
    
    // Initialize database
    let db = init_database(config).await?;
    
    println!("Connected to {:?}", db.db_type());
    println!("Database name: {}", db.db_name());
    
    // Test connection
    db.ping()?;
    println!("Connection successful!");
    
    Ok(())
}
```

---

## Model Definition with Flexible IDs

### For MongoDB

```rust
use database::utils::{ObjectId, DatabaseId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: ObjectId,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: ObjectId::new(),
            email: email.into(),
            name: name.into(),
            created_at: chrono::Utc::now(),
        }
    }
}
```

### For PostgreSQL/MySQL (with UUID)

```rust
use uuid::Uuid;
use database::utils::DatabaseId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: Uuid::new_v4(),
            email: email.into(),
            name: name.into(),
            created_at: chrono::Utc::now(),
        }
    }
}
```

### For SQLite (with i64)

```rust
use database::utils::DatabaseId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    pub id: i64,
    pub email: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl User {
    pub fn new(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            id: 0, // Will be auto-assigned by SQLite
            email: email.into(),
            name: name.into(),
            created_at: chrono::Utc::now(),
        }
    }
}
```

---

## Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `DB_TYPE` | Database type | mongodb |
| `DATABASE_URL` | Full connection string | - |
| `MONGODB_URI` | MongoDB connection URI | mongodb://localhost:27017 |
| `POSTGRES_URI` | PostgreSQL connection URI | postgres://localhost/mydb |
| `MYSQL_URI` | MySQL connection URI | mysql://root@localhost/mydb |
| `DATABASE_NAME` | Database name | myapp |

---

## Features

Enable databases in Cargo.toml:

```toml
[dependencies]
database = { path = "../database", features = [
    "mongodb",    # MongoDB support
    "postgres",   # PostgreSQL support
    "mysql",      # MySQL support
    "sqlite",     # SQLite support
] }
```

---

## Error Handling

```rust
use database::utils::{DbError, DbResult};

type DbResult<T> = Result<T, DbError>;

#[derive(Debug)]
pub enum DbError {
    ConnectionFailed(String),
    QueryFailed(String),
    NotFound(String),
    NotSupported(String),
    InvalidInput(String),
}

impl DbError {
    pub fn connection_failed(msg: &str) -> Self {
        DbError::ConnectionFailed(msg.to_string())
    }
    
    pub fn not_found(msg: &str) -> Self {
        DbError::NotFound(msg.to_string())
    }
    
    pub fn not_supported(feature: &str) -> Self {
        DbError::NotSupported(feature.to_string())
    }
}
```

---

## Testing

```bash
# Test all features
cargo test -p database

# Test specific database
cargo test -p database --features mongodb
cargo test -p database --features postgres
```

---

## License

MIT
