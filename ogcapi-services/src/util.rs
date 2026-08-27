use tracing::error;

/// Helper function to read-lock a `RwLock`, recovering from poisoning if necessary.
pub fn read_lock<T>(lock: &std::sync::RwLock<T>) -> std::sync::RwLockReadGuard<'_, T> {
    match lock.read() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("RwLock was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}

/// Helper function to write-lock a `RwLock`, recovering from poisoning if necessary.
pub fn write_lock<T>(lock: &std::sync::RwLock<T>) -> std::sync::RwLockWriteGuard<'_, T> {
    match lock.write() {
        Ok(guard) => guard,
        Err(poisoned) => {
            error!("RwLock was poisoned, attempting to recover.");
            poisoned.into_inner()
        }
    }
}
