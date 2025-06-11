# derive-error-kind

[![Crates.io](https://img.shields.io/crates/v/derive-error-kind.svg)](https://crates.io/crates/derive-error-kind)
[![Documentation](https://docs.rs/derive-error-kind/badge.svg)](https://docs.rs/derive-error-kind)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A Rust procedural macro for implementing the ErrorKind pattern that simplifies error classification and handling in complex applications.

## Motivation

The ErrorKind pattern is a common technique in Rust for separating:
- The **kind** of an error (represented by a simple enum)
- The **details** of the error (contained in the error structure)

This allows developers to handle errors more granularly without losing context.

Rust's standard library uses this pattern in `std::io::ErrorKind`, and many other libraries have adopted it due to its flexibility. However, manually implementing this pattern can be repetitive and error-prone, especially in applications with multiple nested error types.

This crate solves this problem by providing a derive macro that automates the implementation of the ErrorKind pattern.

## Overview

The `ErrorKind` macro allows you to associate error types with a specific kind from an enum. This creates a clean and consistent way to categorize errors in your application, enabling more precise error handling.

Key features:
- Automatically implements a `.kind()` method that returns a categorized error type
- Supports nested error types via the `transparent` attribute
- Works with unit variants, named fields, and tuple variants
- Enables transparent error propagation through error hierarchies

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
derive-error-kind = "0.1.0"
```

## Basic Usage

First, define an enum for your error kinds:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    NotFound,
    InvalidInput,
    InternalError,
}
```

Then, use the `ErrorKind` derive macro on your error enums:

```rust
use derive_error_kind::ErrorKind;

#[derive(Debug, ErrorKind)]
#[error_kind(ErrorKind)]
pub enum MyError {
    #[error_kind(ErrorKind, NotFound)]
    ResourceNotFound,

    #[error_kind(ErrorKind, InvalidInput)]
    BadRequest { details: String },

    #[error_kind(ErrorKind, InternalError)]
    ServerError(String),
}

// Now you can use the .kind() method
let error = MyError::ResourceNotFound;
assert_eq!(error.kind(), ErrorKind::NotFound);
```

## Advanced Examples

### Nested Error Types

You can create hierarchical error structures with the `transparent` attribute:

```rust
use derive_error_kind::ErrorKind;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    Database,
    Cache,
    Network,
    Configuration,
}

#[derive(Debug, ErrorKind)]
#[error_kind(ErrorKind)]
pub enum DatabaseError {
    #[error_kind(ErrorKind, Database)]
    Connection,

    #[error_kind(ErrorKind, Database)]
    Query(String),
}

#[derive(Debug, ErrorKind)]
#[error_kind(ErrorKind)]
pub enum CacheError {
    #[error_kind(ErrorKind, Cache)]
    Expired,

    #[error_kind(ErrorKind, Cache)]
    Missing,
}

#[derive(Debug, ErrorKind)]
#[error_kind(ErrorKind)]
pub enum AppError {
    #[error_kind(transparent)]
    Db(DatabaseError),

    #[error_kind(transparent)]
    Cache(CacheError),

    #[error_kind(ErrorKind, Network)]
    Connection,

    #[error_kind(ErrorKind, Configuration)]
    InvalidConfig { field: String, message: String },
}

// The transparent attribute allows the kind to bubble up
let db_error = AppError::Db(DatabaseError::Connection);
assert_eq!(db_error.kind(), ErrorKind::Database);

let cache_error = AppError::Cache(CacheError::Missing);
assert_eq!(cache_error.kind(), ErrorKind::Cache);
```

### Integrating with `thiserror`

The `ErrorKind` derive macro works well with other popular error handling crates:

```rust
use derive_error_kind::ErrorKind;
use thiserror::Error;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    NotFound,
    Unauthorized,
    Internal,
}

#[derive(Debug, Error, ErrorKind)]
#[error_kind(ErrorKind)]
pub enum ApiError {
    #[error("Resource not found: {0}")]
    #[error_kind(ErrorKind, NotFound)]
    NotFound(String),

    #[error("Unauthorized access")]
    #[error_kind(ErrorKind, Unauthorized)]
    Unauthorized,

    #[error("Internal server error: {0}")]
    #[error_kind(ErrorKind, Internal)]
    Internal(String),
}

// Use in error handling
fn process_api_result(result: Result<(), ApiError>) {
    if let Err(err) = result {
        match err.kind() {
            ErrorKind::NotFound => {
                // Handle not found errors
                println!("Resource not found: {}", err);
            },
            ErrorKind::Unauthorized => {
                // Handle authorization errors
                println!("Please log in first");
            },
            ErrorKind::Internal => {
                // Log internal errors
                eprintln!("Internal error: {}", err);
            },
        }
    }
}
```

### Web Application Example

Here's a more complete example for a web application with multiple error domains:

```rust
use derive_error_kind::ErrorKind;
use thiserror::Error;
use std::fmt;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum ApiErrorKind {
    Authentication,
    Authorization,
    NotFound,
    BadRequest,
    ServerError,
}

// Implement Display for HTTP status code mapping
impl fmt::Display for ApiErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Authentication => write!(f, "Authentication Failed"),
            Self::Authorization => write!(f, "Not Authorized"),
            Self::NotFound => write!(f, "Resource Not Found"),
            Self::BadRequest => write!(f, "Bad Request"),
            Self::ServerError => write!(f, "Internal Server Error"),
        }
    }
}

// Implement status code conversion
impl ApiErrorKind {
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Authentication => 401,
            Self::Authorization => 403,
            Self::NotFound => 404,
            Self::BadRequest => 400,
            Self::ServerError => 500,
        }
    }
}

// Database errors
#[derive(Debug, Error, ErrorKind)]
#[error_kind(ApiErrorKind)]
pub enum DbError {
    #[error("Database connection failed: {0}")]
    #[error_kind(ApiErrorKind, ServerError)]
    Connection(String),

    #[error("Query execution failed: {0}")]
    #[error_kind(ApiErrorKind, ServerError)]
    Query(String),

    #[error("Entity not found: {0}")]
    #[error_kind(ApiErrorKind, NotFound)]
    NotFound(String),
}

// Auth errors
#[derive(Debug, Error, ErrorKind)]
#[error_kind(ApiErrorKind)]
pub enum AuthError {
    #[error("Invalid credentials")]
    #[error_kind(ApiErrorKind, Authentication)]
    InvalidCredentials,

    #[error("Token expired")]
    #[error_kind(ApiErrorKind, Authentication)]
    TokenExpired,

    #[error("Insufficient permissions for {0}")]
    #[error_kind(ApiErrorKind, Authorization)]
    InsufficientPermissions(String),
}

// Application errors that can wrap domain-specific errors
#[derive(Debug, Error, ErrorKind)]
#[error_kind(ApiErrorKind)]
pub enum AppError {
    #[error(transparent)]
    #[error_kind(transparent)]
    Database(#[from] DbError),

    #[error(transparent)]
    #[error_kind(transparent)]
    Auth(#[from] AuthError),

    #[error("Invalid input: {0}")]
    #[error_kind(ApiErrorKind, BadRequest)]
    InvalidInput(String),

    #[error("Unexpected error: {0}")]
    #[error_kind(ApiErrorKind, ServerError)]
    Unexpected(String),
}

// Example API response
#[derive(Debug, serde::Serialize)]
pub struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ErrorResponse>,
}

#[derive(Debug, serde::Serialize)]
pub struct ErrorResponse {
    code: u16,
    message: String,
}

// Use in a web handler (example with actix-web)
fn handle_error(err: AppError) -> HttpResponse {
    let status_code = err.kind().status_code();

    let response = ApiResponse {
        success: false,
        data: None,
        error: Some(ErrorResponse {
            code: status_code,
            message: err.to_string(),
        }),
    };

    HttpResponse::build(StatusCode::from_u16(status_code).unwrap())
        .json(response)
}

// Usage example
async fn get_user(user_id: String) -> Result<User, AppError> {
    let user = db::find_user(&user_id).await
        .map_err(AppError::Database)?;

    if !user.is_active {
        return Err(AppError::Auth(AuthError::InsufficientPermissions("inactive user".to_string())));
    }

    Ok(user)
}
```

## Benefits

- **Simplified Error Handling**: Map complex errors to a simple enum for clean error handling
- **Better Error Classification**: Categorize errors consistently across your application
- **Cleaner APIs**: Hide implementation details behind error kinds
- **Integration with Error Handling Libraries**: Works well with `thiserror`, `anyhow`, and other error handling crates

## Attribute Reference

- `#[error_kind(KindEnum)]`: Top-level attribute that specifies which enum to use for error kinds
- `#[error_kind(KindEnum, Variant)]`: Variant-level attribute that specifies which variant of the kind enum to return
- `#[error_kind(transparent)]`: Variant-level attribute for nested errors, indicating that the inner error's kind should be used

## Requirements

- The macro can only be applied to enums
- Each variant must have an `error_kind` attribute
- The kind enum must be in scope and accessible

### Microservices Example

Here's an example showing how `derive-error-kind` can be used in a microservices architecture:

```rust
use derive_error_kind::ErrorKind;
use thiserror::Error;
use std::fmt;

// Global error kinds that are consistent across all services
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum GlobalErrorKind {
    // Infrastructure errors
    DatabaseError,
    CacheError,
    NetworkError,

    // Business logic errors
    ValidationError,
    NotFoundError,
    ConflictError,

    // Security errors
    AuthenticationError,
    AuthorizationError,

    // General errors
    ConfigurationError,
    InternalError,
}

// User service errors
#[derive(Debug, Error, ErrorKind)]
#[error_kind(GlobalErrorKind)]
pub enum UserServiceError {
    #[error("Failed to connect to users database: {0}")]
    #[error_kind(GlobalErrorKind, DatabaseError)]
    Database(String),

    #[error("User not found: {0}")]
    #[error_kind(GlobalErrorKind, NotFoundError)]
    NotFound(String),

    #[error("Email already exists: {0}")]
    #[error_kind(GlobalErrorKind, ConflictError)]
    DuplicateEmail(String),
}

// Inventory service errors
#[derive(Debug, Error, ErrorKind)]
#[error_kind(GlobalErrorKind)]
pub enum InventoryServiceError {
    #[error("Failed to connect to inventory database: {0}")]
    #[error_kind(GlobalErrorKind, DatabaseError)]
    Database(String),

    #[error("Product not found: {0}")]
    #[error_kind(GlobalErrorKind, NotFoundError)]
    ProductNotFound(String),

    #[error("Insufficient stock for product: {0}")]
    #[error_kind(GlobalErrorKind, ConflictError)]
    InsufficientStock(String),
}

// Order service errors
#[derive(Debug, Error, ErrorKind)]
#[error_kind(GlobalErrorKind)]
pub enum OrderServiceError {
    #[error("Database error: {0}")]
    #[error_kind(GlobalErrorKind, DatabaseError)]
    Database(String),

    #[error(transparent)]
    #[error_kind(transparent)]
    User(#[from] UserServiceError),

    #[error(transparent)]
    #[error_kind(transparent)]
    Inventory(#[from] InventoryServiceError),

    #[error("Order validation failed: {0}")]
    #[error_kind(GlobalErrorKind, ValidationError)]
    Validation(String),
}

// API Gateway error handling
fn handle_service_error<E: std::error::Error + 'static>(err: E) -> HttpResponse {
    // Use downcast to check if we have an error with a kind() method
    if let Some(user_err) = err.downcast_ref::<UserServiceError>() {
        match user_err.kind() {
            GlobalErrorKind::NotFoundError => return HttpResponse::NotFound().finish(),
            GlobalErrorKind::ConflictError => return HttpResponse::Conflict().finish(),
            _ => { /* continue with other error types */ }
        }
    }

    if let Some(order_err) = err.downcast_ref::<OrderServiceError>() {
        // Here, transparent errors from other services are automatically
        // mapped to the correct GlobalErrorKind
        match order_err.kind() {
            GlobalErrorKind::ValidationError => return HttpResponse::BadRequest().finish(),
            GlobalErrorKind::NotFoundError => return HttpResponse::NotFound().finish(),
            GlobalErrorKind::DatabaseError => {
                log::error!("Database error: {}", order_err);
                return HttpResponse::InternalServerError().finish();
            },
            _ => { /* continue with general error handling */ }
        }
    }

    // Default error response
    HttpResponse::InternalServerError().finish()
}
```


## Best Practices

1. **Keep error categories (ErrorKind) simple and stable**
   - They should change less frequently than your detailed error types

2. **Use the same error category throughout your application**
   - Makes consistent error handling at the API layer easier

3. **Combine with `thiserror` for detailed error messages**
   - `derive-error-kind` handles categorization while `thiserror` handles messages

4. **Use `transparent` for nested errors**
   - Allows the correct error category to propagate automatically

## Acknowledgements

- This project was inspired by the [enum-kinds](https://crates.io/crates/enum-kinds) crate

## License

Licensed under MIT license.
