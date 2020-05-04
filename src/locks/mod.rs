use std::ops::{Deref, DerefMut};

use crate::Result;

cfg_if::cfg_if! {
    if #[cfg(target_os = "windows")] {
        mod windows;
        use windows as os;
    } else if #[cfg(any(target_os = "linux"))] {
        mod unix;
        use unix as os;
    } else {
        unimplemented!("This crate does not support your OS yet !");
    }
}

pub use os::*;

/// Used to wrap an acquired lock's data. Lock is automatically released on `Drop`
pub struct LockGuard<'t> {
    lock: &'t dyn LockImpl,
}
impl<'t> Drop for LockGuard<'t> {
    fn drop(&mut self) {
        self.lock.release().unwrap();
    }
}
impl<'t> LockGuard<'t> {
    fn new(lock_impl: &'t dyn LockImpl) -> Self {
        Self { lock: lock_impl }
    }
    pub fn into_read_guard(self) -> ReadLockGuard<'t> {
        let inner_lock = self.lock;
        std::mem::forget(self);
        ReadLockGuard::new(inner_lock)
    }
}
impl<'t> Deref for LockGuard<'t> {
    type Target = *mut u8;
    fn deref(&self) -> &Self::Target {
        unsafe { self.lock.get_inner() }
    }
}
impl<'t> DerefMut for LockGuard<'t> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { self.lock.get_inner() }
    }
}

/// Used to wrap an acquired lock's read only data. Lock is automatically released on `Drop`
pub struct ReadLockGuard<'t> {
    lock: &'t dyn LockImpl,
}
impl<'t> ReadLockGuard<'t> {
    fn new(lock_impl: &'t dyn LockImpl) -> Self {
        Self { lock: lock_impl }
    }
}

impl<'t> Drop for ReadLockGuard<'t> {
    fn drop(&mut self) {
        self.lock.release().unwrap();
    }
}
impl<'t> Deref for ReadLockGuard<'t> {
    type Target = *const u8;
    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.lock.get_inner() as *mut *mut u8 as *const *const u8) }
    }
}

pub trait LockInit {
    /// Size required for the lock's internal representation
    fn size_of() -> usize;
    /// Potential Alignment requirements for the lock's internal representation
    fn alignment() -> Option<u8>;
    /// Initializes a new instance of the lock in the provided buffer and returns the number of used bytes
    unsafe fn new(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)>;
    /// Re-uses a lock from an already initialized location and returns the number of used bytes
    unsafe fn from_existing(mem: *mut u8, data: *mut u8) -> Result<(Box<dyn LockImpl>, usize)>;
}

pub trait LockImpl {
    /// Acquires the lock
    fn lock(&self) -> Result<LockGuard<'_>>;
    /// Release the lock
    fn release(&self) -> Result<()>;

    /// Acquires the lock for read access only. This method uses `lock()` as a fallback
    fn rlock(&self) -> Result<ReadLockGuard<'_>> {
        Ok(self.lock()?.into_read_guard())
    }

    /// Leaks the inner data without acquiring the lock
    #[doc(hidden)]
    unsafe fn get_inner(&self) -> &mut *mut u8;
}
