use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CancellationToken {
    task_id: String,
    cancelled: Arc<AtomicBool>,
}

#[allow(dead_code)]
impl CancellationToken {
    pub fn new(task_id: impl Into<String>) -> Self {
        Self {
            task_id: task_id.into(),
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn task_id(&self) -> &str {
        &self.task_id
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }

    pub fn throw_if_cancelled(&self) -> Result<(), String> {
        if self.is_cancelled() {
            return Err(format!("Task {} was cancelled.", self.task_id));
        }

        Ok(())
    }
}

pub trait CancellableProcessHandle: Send + Sync {
    fn abort(&self) -> Result<(), String>;
}

#[derive(Default)]
pub struct ProcessHandleRegistry {
    handles: Mutex<HashMap<String, Vec<Arc<dyn CancellableProcessHandle>>>>,
}

#[allow(dead_code)]
impl ProcessHandleRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(
        &self,
        task_id: impl Into<String>,
        handle: Arc<dyn CancellableProcessHandle>,
    ) -> Result<(), String> {
        let mut handles = self
            .handles
            .lock()
            .map_err(|_| "Process handle registry lock was poisoned.".to_string())?;
        handles.entry(task_id.into()).or_default().push(handle);
        Ok(())
    }

    pub fn abort_task(&self, task_id: &str) -> Result<usize, String> {
        let handles = self
            .handles
            .lock()
            .map_err(|_| "Process handle registry lock was poisoned.".to_string())?
            .remove(task_id)
            .unwrap_or_default();

        let mut aborted_count = 0;
        for handle in handles {
            handle.abort()?;
            aborted_count += 1;
        }

        Ok(aborted_count)
    }
}

#[cfg(test)]
mod tests {
    use super::{CancellableProcessHandle, CancellationToken, ProcessHandleRegistry};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    struct FakeHandle {
        abort_count: Arc<AtomicUsize>,
    }

    impl CancellableProcessHandle for FakeHandle {
        fn abort(&self) -> Result<(), String> {
            self.abort_count.fetch_add(1, Ordering::SeqCst);
            Ok(())
        }
    }

    #[test]
    fn cancellation_token_stops_at_safe_points() {
        let token = CancellationToken::new("task_cancel_token");

        assert!(!token.is_cancelled());
        token
            .throw_if_cancelled()
            .expect("not cancelled token should pass");

        token.cancel();
        assert!(token.is_cancelled());
        assert!(token.throw_if_cancelled().is_err());
    }

    #[test]
    fn process_registry_aborts_registered_handles_once() {
        let registry = ProcessHandleRegistry::new();
        let abort_count = Arc::new(AtomicUsize::new(0));

        registry
            .register(
                "task_abort",
                Arc::new(FakeHandle {
                    abort_count: Arc::clone(&abort_count),
                }),
            )
            .expect("handle should register");

        let aborted = registry
            .abort_task("task_abort")
            .expect("task handles should abort");
        assert_eq!(aborted, 1);
        assert_eq!(abort_count.load(Ordering::SeqCst), 1);

        let aborted_again = registry
            .abort_task("task_abort")
            .expect("duplicate abort should be idempotent");
        assert_eq!(aborted_again, 0);
        assert_eq!(abort_count.load(Ordering::SeqCst), 1);
    }
}
