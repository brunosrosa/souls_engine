//! Dedicated OS Worker Thread harness with panic isolation and timeout bounds.

use std::time::Duration;
use crate::error::ArenaError;

/// Dispatches a compute-heavy or FFI-bound operation to a dedicated OS thread.
///
/// Ensures:
/// 1. Panics are trapped safely via `catch_unwind` without tearing down the daemon.
/// 2. Timeouts are enforced deterministically via cross-thread channels.
pub fn dispatch_safe_worker<F, T>(
    name: &str,
    timeout: Duration,
    task: F,
) -> Result<T, ArenaError>
where
    F: FnOnce() -> Result<T, ArenaError> + Send + 'static,
    T: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let thread_name = format!("souls-arena-{}", name);

    let handle = std::thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            let res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(task));
            let outcome = match res {
                Ok(inner_res) => inner_res,
                Err(payload) => {
                    let msg = if let Some(s) = payload.downcast_ref::<&str>() {
                        s.to_string()
                    } else if let Some(s) = payload.downcast_ref::<String>() {
                        s.clone()
                    } else {
                        "Unknown panic in worker thread".to_string()
                    };
                    Err(ArenaError::WorkerPanic(msg))
                }
            };
            let _ = tx.send(outcome);
        });

    if handle.is_err() {
        return Err(ArenaError::WorkerPanic("Failed to spawn dedicated OS thread".to_string()));
    }

    match rx.recv_timeout(timeout) {
        Ok(res) => res,
        Err(_) => Err(ArenaError::Timeout(format!(
            "Worker '{}' exceeded timeout limit of {:?}",
            name, timeout
        ))),
    }
}
