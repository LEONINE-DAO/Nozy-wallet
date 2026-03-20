# Error Handling

NozyWallet uses a comprehensive error handling system with user-friendly error messages.

## Error Types

The `NozyError` enum covers all error cases:

```rust
use nozy::{NozyError, NozyResult};

match result {
    Ok(value) => println!("Success: {:?}", value),
    Err(NozyError::NetworkError(_)) => {
        println!("Network error: {}", error.user_friendly_message());
    },
    Err(NozyError::AddressParsing(_)) => {
        println!("Address error: {}", error.user_friendly_message());
    },
    Err(e) => println!("Other error: {}", e),
}
```

## User-Friendly Messages

All errors provide user-friendly messages with suggestions:

```rust
match error {
    NozyError::NetworkError(_) => {
        // Suggests checking Zebra connection
    },
    NozyError::InsufficientFunds { available, required } => {
        // Shows available vs required amounts
    },
    // ... other error types
}
```

## Result Type

All API functions return `NozyResult<T>` which is an alias for `Result<T, NozyError>`:

```rust
use nozy::NozyResult;

fn example() -> NozyResult<String> {
    // Your code here
    Ok("success".to_string())
}
```
