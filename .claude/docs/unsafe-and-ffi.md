# Unsafe Code & FFI

## When Unsafe is Valid

| Use Case           | Example                                            |
| ------------------ | -------------------------------------------------- |
| FFI                | Calling C functions                                |
| Novel abstractions | Implementing `Vec`, `Arc`                          |
| Performance        | Measured bottleneck with safe alternative too slow |

**NOT valid:** Escaping borrow checker without understanding why, bypassing `Send`/`Sync` bounds, shortening safe Rust via `transmute`.

`unsafe` implies risk of undefined behavior. Do not use it to mark functions that are merely "dangerous" for other reasons.

## Required Documentation

```rust
// SAFETY: <why this specific usage is safe>
unsafe { ... }

/// # Safety
/// <what the caller must guarantee>
pub unsafe fn dangerous() { ... }
```

SAFETY comments must be specific and verifiable — not vague ("this is safe").

## Safety Invariants Checklist

Before every unsafe block:

- [ ] SAFETY comment present with specific reasoning
- [ ] Pointer validity: non-null, aligned, initialized, valid for duration of use
- [ ] No aliasing violations (`&` and `&mut` never to same memory simultaneously)
- [ ] Panic safety: data structures left valid on panic; panic guard if needed
- [ ] Tested with Miri (`cargo miri test`)

## Quick Reference

| Operation        | Requirements                                 |
| ---------------- | -------------------------------------------- |
| `*ptr` deref     | Valid, aligned, initialized                  |
| `&*ptr`          | + No aliasing violations                     |
| `transmute`      | Same size, valid bit pattern                 |
| `extern "C"`     | Correct signature, ABI                       |
| `static mut`     | Don't use; use atomics/sync instead          |
| `impl Send/Sync` | Actually thread-safe; expert review required |
| `set_len` on Vec | All elements must be initialized             |
| `from_raw_parts` | Valid pointer, correct lifetime              |

## Common Pitfalls

| Pitfall                                   | Fix                                      |
| ----------------------------------------- | ---------------------------------------- |
| Dangling pointer from local               | Heap allocate or return value            |
| `CString::new().unwrap().as_ptr()`        | Store `CString` in variable first        |
| `Vec::set_len` with uninitialized data    | Use `push`/`resize` or `MaybeUninit`     |
| Reference to `repr(packed)` field         | Copy by value or `read_unaligned`        |
| Mutable aliasing through raw pointers     | Use single pointer, sequential access    |
| Invalid enum discriminant via `transmute` | Use `TryFrom` or `match`                 |
| Panic unwinding across FFI                | Wrap in `catch_unwind`                   |
| Double free from Clone + Drop             | Don't implement Clone for owning handles |

## FFI Patterns

### Safe Wrapper Structure

```rust
mod ffi {
    unsafe extern "C" {
        pub fn lib_create(name: *const c_char) -> *mut c_void;
        pub fn lib_destroy(handle: *mut c_void);
    }
}

pub struct Library {
    handle: NonNull<c_void>,
}

impl Library {
    pub fn new(name: &str) -> Result<Self, Error> {
        let c_name = CString::new(name)?;
        let handle = unsafe { ffi::lib_create(c_name.as_ptr()) };
        NonNull::new(handle).map(|h| Self { handle: h }).ok_or(Error)
    }
}

impl Drop for Library {
    fn drop(&mut self) {
        unsafe { ffi::lib_destroy(self.handle.as_ptr()); }
    }
}
```

### Key FFI Rules

1. Always use `#[repr(C)]` for types crossing FFI boundaries
2. Handle null pointers at the boundary
3. Catch panics before returning to C (`catch_unwind`)
4. Document ownership clearly (who allocates, who frees)
5. Use opaque handle types to prevent mixing up handles
6. Use `NonNull<T>` over `*mut T`
7. Use `PhantomData<T>` to track lifetimes and ownership for pointers
8. Prefer `pointer::cast()` over `as` for pointer casting
9. Use `bindgen` (C→Rust) and `cbindgen` (Rust→C) for bindings
10. Verify layout at compile time with const assertions

### Edition 2024 FFI Changes

```rust
// extern blocks require `unsafe` keyword
unsafe extern "C" {
    fn external_func();
}

// Unsafe attributes require wrapping
#[unsafe(no_mangle)]
pub fn exported() {}

// Unsafe ops in unsafe fns require explicit blocks
unsafe fn do_stuff(ptr: *const i32) -> i32 {
    unsafe { *ptr }
}
```

## Deprecated Unsafe Patterns

| Deprecated                         | Use Instead              |
| ---------------------------------- | ------------------------ |
| `mem::uninitialized()`             | `MaybeUninit<T>`         |
| `mem::zeroed()` for refs           | `MaybeUninit<T>`         |
| Raw pointer arithmetic             | `NonNull<T>`, `ptr::add` |
| `CString::new().unwrap().as_ptr()` | Store CString first      |
| `static mut`                       | `AtomicT` or `Mutex`     |
| Manual extern declarations         | `bindgen`                |

## Review Severity Guide

| Pattern              | Requires                                 |
| -------------------- | ---------------------------------------- |
| `transmute`          | Two reviewers, Miri test                 |
| Manual `Send`/`Sync` | Thread safety expert review              |
| FFI                  | Documentation of C interface             |
| `static mut`         | Justification for not using atomic/mutex |
| Pointer arithmetic   | Bounds proof                             |

## Related Detail Files

- `unsafe-rules-reference.md` — Full 47-rule reference across 7 categories
- `unsafe-checklists.md` — Before-writing, reviewing, and common pitfalls checklists
- `unsafe-examples.md` — 5 safe abstraction patterns + 5 FFI patterns with complete code
