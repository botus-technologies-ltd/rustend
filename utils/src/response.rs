use serde::{Deserialize, Serialize};

/// Standard API response wrapper for consistent responses across the workspace
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ApiError>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<ResponseMeta>,
}

impl<T> ApiResponse<T> {
    /// Create a successful response with data
    pub fn success_data(message: impl Into<String>, data: T) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            error: None,
            meta: None,
        }
    }

    /// Create a successful response without data
    pub fn ok(message: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            error: None,
            meta: None,
        }
    }

    /// Create an error response
    pub fn error(message: impl Into<String>, error: Option<ApiError>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            error,
            meta: None,
        }
    }

    /// Create a validation error response
    pub fn validation_error(errors: Vec<ValidationError>) -> Self {
        Self {
            success: false,
            message: "Validation failed".into(),
            data: None,
            error: Some(ApiError {
                code: "VALIDATION_ERROR".into(),
                details: Some(serde_json::json!(errors)),
            }),
            meta: None,
        }
    }

    /// Add pagination metadata to response
    pub fn with_meta(mut self, meta: ResponseMeta) -> Self {
        self.meta = Some(meta);
        self
    }
}

/// API Error structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ApiError {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// Validation error for form/API input
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ValidationError {
    pub field: String,
    pub message: String,
}

/// Pagination metadata
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ResponseMeta {
    pub page: u32,
    pub per_page: u32,
    pub total: u64,
    pub total_pages: u32,
}

impl ResponseMeta {
    pub fn new(page: u32, per_page: u32, total: u64) -> Self {
        let total_pages = (total as f64 / per_page as f64).ceil() as u32;
        Self {
            page,
            per_page,
            total,
            total_pages,
        }
    }
}

/// Paginated response helper
#[derive(Debug, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Vec<T>,
    pub meta: ResponseMeta,
}

impl<T> PaginatedResponse<T> {
    pub fn new(message: impl Into<String>, data: Vec<T>, meta: ResponseMeta) -> Self {
        Self {
            success: true,
            message: message.into(),
            data,
            meta,
        }
    }
}
