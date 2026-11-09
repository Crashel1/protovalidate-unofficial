use pyo3::prelude::*;
use std::sync::{LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard, TryLockError};
// Release the GIL while waiting so a Python-preempted lock owner can finish.
pub(crate) trait RwLockExt<T: Send + Sync> {
    fn read_py_attached(&self, py: Python<'_>) -> LockResult<RwLockReadGuard<'_, T>>;
    fn write_py_attached(&self, py: Python<'_>) -> LockResult<RwLockWriteGuard<'_, T>>;
}
impl<T: Send + Sync> RwLockExt<T> for RwLock<T> {
    fn read_py_attached(&self, py: Python<'_>) -> LockResult<RwLockReadGuard<'_, T>> {
        loop {
            match self.try_read() {
                Ok(guard) => return Ok(guard),
                Err(TryLockError::Poisoned(error)) => return Err(error),
                Err(TryLockError::WouldBlock) => py.detach(|| drop(self.read())),
            }
        }
    }
    fn write_py_attached(&self, py: Python<'_>) -> LockResult<RwLockWriteGuard<'_, T>> {
        loop {
            match self.try_write() {
                Ok(guard) => return Ok(guard),
                Err(TryLockError::Poisoned(error)) => return Err(error),
                Err(TryLockError::WouldBlock) => py.detach(|| drop(self.write())),
            }
        }
    }
}
