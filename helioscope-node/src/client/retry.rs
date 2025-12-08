use std::future::Future;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

use super::http::ClientError;

pub async fn send_with_retry<F, Fut>(f: F, max_retries: u32) -> Result<(), ClientError>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<(), ClientError>>,
{
    let mut attempt = 0;

    loop {
        attempt += 1;

        match f().await {
            Ok(_) => {
                if attempt > 1 {
                    debug!("Successfully sent after {} attempts", attempt);
                }
                return Ok(());
            }
            Err(e) => {
                if attempt >= max_retries {
                    warn!("Failed to send after {} attempts: {}", max_retries, e);
                    return Err(e);
                }

                // Exponential backoff: 1s, 2s, 4s, 8s...
                let backoff_secs = 2u64.pow(attempt - 1);
                let backoff = Duration::from_secs(backoff_secs);

                warn!(
                    "Attempt {}/{} failed: {}. Retrying in {:?}",
                    attempt, max_retries, e, backoff
                );

                sleep(backoff).await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    #[tokio::test]
    async fn test_succeeds_first_try() {
        let result = send_with_retry(|| async { Ok(()) }, 3).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_succeeds_after_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = send_with_retry(
            move || {
                let c = Arc::clone(&counter_clone);
                async move {
                    let count = c.fetch_add(1, Ordering::SeqCst);
                    if count < 2 {
                        Err(ClientError::Http("temporary failure".to_string()))
                    } else {
                        Ok(())
                    }
                }
            },
            5,
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_fails_after_max_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let result = send_with_retry(
            move || {
                let c = Arc::clone(&counter_clone);
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err(ClientError::Http("permanent failure".to_string()))
                }
            },
            3,
        )
        .await;

        assert!(result.is_err());
        assert_eq!(counter.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn test_respects_max_retries() {
        let counter = Arc::new(AtomicU32::new(0));
        let counter_clone = Arc::clone(&counter);

        let _result = send_with_retry(
            move || {
                let c = Arc::clone(&counter_clone);
                async move {
                    c.fetch_add(1, Ordering::SeqCst);
                    Err(ClientError::Http("always fails".to_string()))
                }
            },
            5,
        )
        .await;

        // Should attempt exactly max_retries times
        assert_eq!(counter.load(Ordering::SeqCst), 5);
    }
}
