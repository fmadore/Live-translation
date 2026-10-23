use std::sync::{Mutex, MutexGuard};

/// Lock a mutex, taking it over if a panicking holder poisoned it. Every lock in the crate
/// guards state that stays consistent between statements (a handle, a counter, a file the next
/// writer replaces whole), so a panic elsewhere is no reason to stop the app from using it.
pub(crate) fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
