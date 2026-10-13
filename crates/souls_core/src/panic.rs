//! Asynchronous and synchronous panic containment barriers for Souls Engine (v7).
//!
//! Envelopes Tokio task futures and synchronous closures to prevent isolated panics
//! from bringing down the bare-metal daemon process.
//!
//! Conforms to CRATES_SPECIFICATION.md Section 2.2.

use crate::error::CoreError;
use std::future::Future;
use std::panic::{catch_unwind, AssertUnwindSafe};

/// Executes an asynchronous future within an isolated Tokio task barrier.
///
/// If the task panics, the panic is caught and converted into a `CoreError::InternalPanic`.
/// Normal errors are preserved.
pub async fn run_safe_task<F, T>(future: F) -> Result<T, CoreError>
where
    F: Future<Output = T> + Send + 'static,
    T: Send + 'static,
{
    let handle = tokio::task::spawn(future);
    match handle.await {
        Ok(result) => Ok(result),
        Err(join_err) => {
            if join_err.is_panic() {
                tracing::error!(
                    target: "souls_core::panic",
                    "Caught panic in spawned Tokio task: {:?}",
                    join_err
                );
                Err(CoreError::InternalPanic(
                    "Async task panicked within Tokio barrier".to_string(),
                ))
            } else {
                Err(CoreError::TaskJoinError(join_err.to_string()))
            }
        }
    }
}

/// Executes a synchronous closure within a `catch_unwind` barrier.
///
/// Converts any unwind panic payload into `CoreError::InternalPanic`.
pub fn run_safe_sync<F, T>(closure: F) -> Result<T, CoreError>
where
    F: FnOnce() -> T,
{
    match catch_unwind(AssertUnwindSafe(closure)) {
        Ok(res) => Ok(res),
        Err(err) => {
            let msg = if let Some(s) = err.downcast_ref::<&str>() {
                (*s).to_string()
            } else if let Some(s) = err.downcast_ref::<String>() {
                s.clone()
            } else {
                "Unknown panic payload".to_string()
            };
            tracing::error!(
                target: "souls_core::panic",
                "Caught synchronous panic in execution barrier: {}",
                msg
            );
            Err(CoreError::InternalPanic(msg))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_run_safe_task_success() {
        let res = run_safe_task(async { 42 }).await;
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_run_safe_task_catches_panic() {
        let res = run_safe_task(async {
            panic!("Boom in async task!");
        })
        .await;

        assert!(res.is_err());
        match res.unwrap_err() {
            CoreError::InternalPanic(msg) => {
                assert!(msg.contains("Async task panicked"));
            }
            other => panic!("Expected InternalPanic, got: {:?}", other),
        }
    }

    #[test]
    fn test_run_safe_sync_success() {
        let res = run_safe_sync(|| "hello".to_string());
        assert!(res.is_ok());
        assert_eq!(res.unwrap(), "hello");
    }

    #[test]
    fn test_run_safe_sync_catches_panic() {
        let res = run_safe_sync(|| {
            panic!("Intentional sync panic!");
        });

        assert!(res.is_err());
        match res.unwrap_err() {
            CoreError::InternalPanic(msg) => {
                assert!(msg.contains("Intentional sync panic!"));
            }
            other => panic!("Expected InternalPanic, got: {:?}", other),
        }
    }
}
