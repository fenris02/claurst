# Safe Abstraction & FFI Pattern Examples

## Safe Abstraction Patterns

### Pattern 1: Wrapper with Bounds Check

```rust
/// A slice wrapper providing unchecked access internally, safe access externally.
pub struct SafeSlice<'a, T> {
    ptr: *const T,
    len: usize,
    _marker: std::marker::PhantomData<&'a T>,
}

impl<'a, T> SafeSlice<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        Self {
            ptr: slice.as_ptr(),
            len: slice.len(),
            _marker: std::marker::PhantomData,
        }
    }

    pub fn get(&self, index: usize) -> Option<&T> {
        if index < self.len {
            // SAFETY: We just verified index < len
            Some(unsafe { &*self.ptr.add(index) })
        } else {
            None
        }
    }

    /// # Safety
    /// `index` must be less than `self.len()`.
    pub unsafe fn get_unchecked(&self, index: usize) -> &T {
        debug_assert!(index < self.len);
        &*self.ptr.add(index)
    }

    pub fn len(&self) -> usize {
        self.len
    }
}
```

### Pattern 2: Resource Wrapper with Drop

```rust
use std::ptr::NonNull;

/// Safe wrapper around a C-allocated buffer.
pub struct CBuffer {
    ptr: NonNull<u8>,
    len: usize,
}

unsafe extern "C" {
    fn c_alloc(size: usize) -> *mut u8;
    fn c_free(ptr: *mut u8);
}

impl CBuffer {
    pub fn new(size: usize) -> Option<Self> {
        let ptr = unsafe { c_alloc(size) };
        NonNull::new(ptr).map(|ptr| Self { ptr, len: size })
    }

    pub fn as_slice(&self) -> &[u8] {
        // SAFETY: ptr is valid for len bytes (from c_alloc contract)
        unsafe { std::slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        // SAFETY: We have &mut self, so exclusive access
        unsafe { std::slice::from_raw_parts_mut(self.ptr.as_ptr(), self.len) }
    }
}

impl Drop for CBuffer {
    fn drop(&mut self) {
        // SAFETY: ptr was allocated by c_alloc and not yet freed
        unsafe { c_free(self.ptr.as_ptr()); }
    }
}

// CBuffer is not Clone/Copy — prevents double-free

// Safe to send between threads (assuming c_alloc is thread-safe)
unsafe impl Send for CBuffer {}
```

### Pattern 3: Interior Mutability with UnsafeCell

```rust
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, Ordering};

/// A simple spinlock demonstrating safe abstraction over UnsafeCell.
pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(data),
        }
    }

    pub fn lock(&self) -> SpinLockGuard<'_, T> {
        while self.locked.compare_exchange_weak(
            false, true, Ordering::Acquire, Ordering::Relaxed,
        ).is_err() {
            std::hint::spin_loop();
        }
        SpinLockGuard { lock: self }
    }
}

impl<T> std::ops::Deref for SpinLockGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: We hold the lock, so we have exclusive access
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> std::ops::DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: We hold the lock, so we have exclusive access
        unsafe { &mut *self.lock.data.get() }
    }
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.locked.store(false, Ordering::Release);
    }
}

// SAFETY: The lock ensures only one thread accesses data at a time
unsafe impl<T: Send> Sync for SpinLock<T> {}
unsafe impl<T: Send> Send for SpinLock<T> {}
```

### Pattern 4: Iterator with Lifetime Tracking

```rust
use std::marker::PhantomData;

/// An iterator over raw pointer range with proper lifetime tracking.
pub struct PtrIter<'a, T> {
    current: *const T,
    end: *const T,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> PtrIter<'a, T> {
    pub fn new(slice: &'a [T]) -> Self {
        let ptr = slice.as_ptr();
        Self {
            current: ptr,
            // SAFETY: Adding len to slice pointer is always valid
            end: unsafe { ptr.add(slice.len()) },
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for PtrIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current == self.end {
            None
        } else {
            // SAFETY:
            // - current < end (checked above)
            // - PhantomData<&'a T> ensures the data lives for 'a
            let item = unsafe { &*self.current };
            self.current = unsafe { self.current.add(1) };
            Some(item)
        }
    }
}
```

### Pattern 5: Builder with Delayed Initialization

```rust
use std::mem::MaybeUninit;

/// A builder that collects exactly N items, then produces an array.
pub struct ArrayBuilder<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    count: usize,
}

impl<T, const N: usize> ArrayBuilder<T, N> {
    pub fn new() -> Self {
        Self {
            data: [const { MaybeUninit::uninit() }; N],
            count: 0,
        }
    }

    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.count >= N {
            return Err(value);
        }
        self.data[self.count].write(value);
        self.count += 1;
        Ok(())
    }

    pub fn build(mut self) -> Option<[T; N]> {
        if self.count != N {
            return None;
        }
        // SAFETY: All N elements have been initialized
        let result = unsafe {
            let data = std::ptr::read(&raw mut self.data);
            std::mem::forget(self);
            MaybeUninit::array_assume_init(data)
        };
        Some(result)
    }
}

impl<T, const N: usize> Drop for ArrayBuilder<T, N> {
    fn drop(&mut self) {
        // Drop only initialized elements
        for i in 0..self.count {
            // SAFETY: Elements 0..count are initialized
            unsafe { self.data[i].assume_init_drop(); }
        }
    }
}
```

### Key Abstraction Principles

1. **Encapsulation**: Hide unsafe behind safe public API
2. **Invariant maintenance**: Use private fields to maintain invariants
3. **PhantomData**: Track lifetimes and ownership for pointers
4. **RAII**: Use Drop for cleanup
5. **Type state**: Use types to encode valid states

---

## FFI Patterns

### Pattern 1: Basic FFI Wrapper

```rust
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_void};
use std::ptr::NonNull;

mod ffi {
    use super::*;

    unsafe extern "C" {
        pub fn lib_create(name: *const c_char) -> *mut c_void;
        pub fn lib_destroy(handle: *mut c_void);
        pub fn lib_process(handle: *mut c_void, data: *const u8, len: usize) -> c_int;
        pub fn lib_get_error() -> *const c_char;
    }
}

pub struct Library {
    handle: NonNull<c_void>,
}

#[derive(Debug)]
pub struct LibraryError(String);

impl Library {
    pub fn new(name: &str) -> Result<Self, LibraryError> {
        let c_name = CString::new(name)
            .map_err(|_| LibraryError("invalid name".into()))?;
        let handle = unsafe { ffi::lib_create(c_name.as_ptr()) };
        NonNull::new(handle)
            .map(|handle| Self { handle })
            .ok_or_else(|| Self::last_error())
    }

    pub fn process(&self, data: &[u8]) -> Result<(), LibraryError> {
        let result = unsafe {
            ffi::lib_process(self.handle.as_ptr(), data.as_ptr(), data.len())
        };
        if result == 0 { Ok(()) } else { Err(Self::last_error()) }
    }

    fn last_error() -> LibraryError {
        let ptr = unsafe { ffi::lib_get_error() };
        if ptr.is_null() {
            LibraryError("unknown error".into())
        } else {
            let msg = unsafe { CStr::from_ptr(ptr) }
                .to_string_lossy()
                .into_owned();
            LibraryError(msg)
        }
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { ffi::lib_destroy(self.handle.as_ptr()); }
    }
}

// Library is not Clone/Copy — prevents double-free
```

### Pattern 2: Callback Registration

```rust
use std::os::raw::{c_int, c_void};
use std::panic::{catch_unwind, AssertUnwindSafe};

type CCallback = extern "C" fn(value: c_int, user_data: *mut c_void) -> c_int;

unsafe extern "C" {
    fn register_callback(cb: CCallback, user_data: *mut c_void);
    fn unregister_callback();
}

/// Safely register a Rust closure as a C callback.
pub struct CallbackGuard<F> {
    _closure: Box<F>,
}

impl<F: FnMut(i32) -> i32 + 'static> CallbackGuard<F> {
    pub fn register(closure: F) -> Self {
        let boxed = Box::new(closure);
        let user_data = Box::into_raw(boxed) as *mut c_void;

        extern "C" fn trampoline<F: FnMut(i32) -> i32>(
            value: c_int,
            user_data: *mut c_void,
        ) -> c_int {
            let result = catch_unwind(AssertUnwindSafe(|| {
                let closure = unsafe { &mut *(user_data as *mut F) };
                closure(value as i32) as c_int
            }));
            result.unwrap_or(-1)
        }

        unsafe { register_callback(trampoline::<F>, user_data); }

        Self {
            // SAFETY: We just created this box and need to keep it alive
            _closure: unsafe { Box::from_raw(user_data as *mut F) },
        }
    }
}

impl<F> Drop for CallbackGuard<F> {
    fn drop(&mut self) {
        unsafe { unregister_callback(); }
    }
}
```

### Pattern 3: Opaque Handle Types

```rust
use std::marker::PhantomData;

// Opaque type markers — prevents mixing up handles
#[repr(C)]
pub struct DatabaseHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}

#[repr(C)]
pub struct ConnectionHandle {
    _data: [u8; 0],
    _marker: PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}

mod ffi {
    use super::*;
    unsafe extern "C" {
        pub fn db_open(path: *const c_char) -> *mut DatabaseHandle;
        pub fn db_close(db: *mut DatabaseHandle);
        pub fn db_connect(db: *mut DatabaseHandle) -> *mut ConnectionHandle;
        pub fn conn_close(conn: *mut ConnectionHandle);
    }
}

pub struct Database {
    handle: NonNull<DatabaseHandle>,
}

pub struct Connection<'db> {
    handle: NonNull<ConnectionHandle>,
    _db: PhantomData<&'db Database>,
}

impl Database {
    pub fn open(path: &str) -> Result<Self, ()> {
        let c_path = CString::new(path).map_err(|_| ())?;
        let handle = unsafe { ffi::db_open(c_path.as_ptr()) };
        NonNull::new(handle).map(|h| Self { handle: h }).ok_or(())
    }

    pub fn connect(&self) -> Result<Connection<'_>, ()> {
        let handle = unsafe { ffi::db_connect(self.handle.as_ptr()) };
        NonNull::new(handle)
            .map(|h| Connection { handle: h, _db: PhantomData })
            .ok_or(())
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        // All Connections must be dropped first (enforced by lifetime)
        unsafe { ffi::db_close(self.handle.as_ptr()); }
    }
}

impl Drop for Connection<'_> {
    fn drop(&mut self) {
        unsafe { ffi::conn_close(self.handle.as_ptr()); }
    }
}
```

### Pattern 4: Error Handling Across FFI

```rust
use std::os::raw::c_int;

pub const SUCCESS: c_int = 0;
pub const ERR_NULL_PTR: c_int = 1;
pub const ERR_INVALID_UTF8: c_int = 2;
pub const ERR_IO: c_int = 3;
pub const ERR_PANIC: c_int = -1;

// Thread-local error storage
thread_local! {
    static LAST_ERROR: std::cell::RefCell<Option<Box<dyn std::error::Error>>> =
        std::cell::RefCell::new(None);
}

fn set_last_error<E: std::error::Error + 'static>(err: E) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(Box::new(err));
    });
}

/// Get the last error message. Caller must free with `free_string`.
#[unsafe(no_mangle)]
pub extern "C" fn get_last_error() -> *mut c_char {
    LAST_ERROR.with(|e| {
        e.borrow()
            .as_ref()
            .map(|err| {
                CString::new(err.to_string())
                    .unwrap_or_else(|_| CString::new("error").unwrap())
                    .into_raw()
            })
            .unwrap_or(std::ptr::null_mut())
    })
}

/// Free a string returned by this library.
#[unsafe(no_mangle)]
pub extern "C" fn free_string(s: *mut c_char) {
    if !s.is_null() {
        // SAFETY: String was created by CString::into_raw
        unsafe { drop(CString::from_raw(s)); }
    }
}

/// Example function with proper error handling.
#[unsafe(no_mangle)]
pub extern "C" fn do_operation(data: *const u8, len: usize) -> c_int {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<(), c_int> {
        if data.is_null() { return Err(ERR_NULL_PTR); }
        let slice = unsafe { std::slice::from_raw_parts(data, len) };
        std::str::from_utf8(slice).map_err(|e| {
            set_last_error(e);
            ERR_INVALID_UTF8
        })?;
        Ok(())
    }));

    match result {
        Ok(Ok(())) => SUCCESS,
        Ok(Err(code)) => code,
        Err(_) => ERR_PANIC,
    }
}
```

### Pattern 5: Struct with C Layout

```rust
use std::os::raw::{c_char, c_int};

#[repr(C)]
pub struct Config {
    pub version: c_int,
    pub flags: u32,
    pub name: [c_char; 64],
    pub name_len: usize,
}

impl Config {
    pub fn new(version: i32, flags: u32, name: &str) -> Option<Self> {
        if name.len() >= 64 { return None; }
        let mut config = Self {
            version: version as c_int,
            flags,
            name: [0; 64],
            name_len: name.len(),
        };
        for (i, byte) in name.bytes().enumerate() {
            config.name[i] = byte as c_char;
        }
        Some(config)
    }

    pub fn name(&self) -> &str {
        let bytes = unsafe {
            std::slice::from_raw_parts(self.name.as_ptr() as *const u8, self.name_len)
        };
        // SAFETY: We only store valid UTF-8 in new()
        unsafe { std::str::from_utf8_unchecked(bytes) }
    }
}

// Verify layout at compile time
const _: () = {
    assert!(std::mem::size_of::<Config>() == 80);
    assert!(std::mem::align_of::<Config>() == 8);
};
```
