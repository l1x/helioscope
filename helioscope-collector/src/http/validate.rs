use std::fmt;

// Validation constant for maximum request size
const MAX_REQUEST_SIZE: usize = 10 * 1024 * 1024; // 10MB

#[derive(Debug)]
pub enum ValidationError {
    RequestTooLarge { size: usize, max: usize },
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RequestTooLarge { size, max } => {
                write!(f, "Request too large: {} bytes (max: {})", size, max)
            }
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validates request body size against maximum allowed size.
///
/// This function checks the actual body size after it has been read.
/// The content-length header check is redundant since we've already read the body.
///
/// # Arguments
///
/// * `body_size` - The actual size of the request body in bytes
///
/// # Returns
///
/// * `Ok(())` if the body size is within limits
/// * `Err(ValidationError::RequestTooLarge)` if the body exceeds the maximum size
pub fn validate_request_size(body_size: usize) -> Result<(), ValidationError> {
    if body_size > MAX_REQUEST_SIZE {
        return Err(ValidationError::RequestTooLarge {
            size: body_size,
            max: MAX_REQUEST_SIZE,
        });
    }

    Ok(())
}

/// Returns the maximum allowed request size in bytes.
///
/// Useful for tests or displaying configuration to operators.
pub fn max_request_size() -> usize {
    MAX_REQUEST_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_request_size_within_limit() {
        assert!(validate_request_size(0).is_ok());
        assert!(validate_request_size(1024).is_ok());
        assert!(validate_request_size(MAX_REQUEST_SIZE).is_ok());
    }

    #[test]
    fn test_validate_request_size_exceeds_limit() {
        let result = validate_request_size(MAX_REQUEST_SIZE + 1);
        assert!(result.is_err());

        match result.unwrap_err() {
            ValidationError::RequestTooLarge { size, max } => {
                assert_eq!(size, MAX_REQUEST_SIZE + 1);
                assert_eq!(max, MAX_REQUEST_SIZE);
            }
        }
    }

    #[test]
    fn test_validate_request_size_boundary() {
        // Test exact boundary
        assert!(validate_request_size(MAX_REQUEST_SIZE).is_ok());
        assert!(validate_request_size(MAX_REQUEST_SIZE + 1).is_err());
    }

    #[test]
    fn test_max_request_size_getter() {
        assert_eq!(max_request_size(), MAX_REQUEST_SIZE);
        assert_eq!(max_request_size(), 10 * 1024 * 1024);
    }

    #[test]
    fn test_error_display() {
        let error = ValidationError::RequestTooLarge {
            size: 20_000_000,
            max: MAX_REQUEST_SIZE,
        };
        let error_string = error.to_string();
        assert!(error_string.contains("20000000"));
        assert!(error_string.contains("10485760"));
    }
}
