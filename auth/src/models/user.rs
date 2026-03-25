//! User model
use chrono::{DateTime, Utc, Duration};
use database::utils::ObjectId;
use serde::{Deserialize, Serialize};

/// User model - stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id:             ObjectId,
    pub email:          Option<String>,
    pub password_hash:  String,

    pub phone:          Option<String>,
    pub username:       Option<String>,
    #[serde(rename =    "firstName")]
    pub first_name:     Option<String>,
     #[serde(rename =   "lastName")]
    pub last_name:      Option<String>,

    #[serde(rename =    "isActive")]
    pub is_active:      bool,
    
    #[serde(rename =    "isVerified")]
    pub is_verified:    bool,
    pub login_attempts: i32,
    pub locked_until:   Option<DateTime<Utc>>,
    pub created_at:     DateTime<Utc>,
    pub updated_at:     Option<DateTime<Utc>,>,
    pub last_login:     Option<DateTime<Utc>,>,
}

impl User {
    pub fn is_locked(&self) -> bool {
        if let Some(locked_until) = self.locked_until {
            return locked_until > chrono::Utc::now();
        }
        false
    }

    pub fn is_verfied(&self) -> bool {
        self.is_verified
    }

    pub fn  is_active(&self) -> bool{
        self.is_active
    }

    pub fn verify_user(&mut self){
        self.is_verified = true
    } 

    pub fn can_attempt_login(&self) -> bool {
        !self.is_active || self.is_locked()
    }

    pub fn lock_user(&mut self, lock_threshold: i32, lock_duration_secs: i64) {
        self.login_attempts += 1;

        if self.login_attempts >= lock_threshold {
            self.locked_until = Some(chrono::Utc::now() + Duration::seconds(lock_duration_secs));
        }
    }

    pub fn reset_failed_attempts(&mut self) {
        self.login_attempts = 0;
        self.locked_until = None;
    }
}


/// User creation input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub email:    Option<String>,
    pub phone:    Option<String>,
    pub password: String,
}

/// User update input
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateUserInput {
    pub email:      Option<String>,
    pub phone:      Option<String>,
    pub username:   Option<String>,
    pub first_name: Option<String>,
    pub last_name:  Option<String>,
}
