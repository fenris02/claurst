# Unsafe Rust Rules Reference

Complete reference of 47 rules across 7 categories for writing and reviewing unsafe Rust code.

---

## General (3 rules)

### general-01: Do Not Abuse Unsafe to Escape Compiler Safety Checks

**Impact: CRITICAL**

Unsafe should not bypass the borrow checker. The borrow checker prevents memory safety bugs; bypassing it defeats Rust's guarantees.

```rust
// BAD: Using unsafe to create aliasing mutable references
let ptr = data.as_mut_ptr();
unsafe {
    let ref1 = &mut *ptr;
    let ref2 = &mut *ptr;  // UB: Two mutable references!
}

// GOOD: Work with the borrow checker
data[0] = 10;
data[0] = 20;  // Sequential mutations are fine

// GOOD: Use interior mutability when needed
let data = RefCell::new(vec![1, 2, 3]);
data.borrow_mut()[0] = 10;
```

**Legitimate uses**: FFI, low-level abstractions (collections, sync primitives), performance (only after profiling).

### general-02: Do Not Blindly Use Unsafe for Performance

**Impact: CRITICAL**

Modern Rust optimizers often eliminate bounds checks when they can prove safety. Unsafe may even prevent optimizations by breaking aliasing assumptions.

```rust
// BAD: Unnecessary unsafe
fn sum_bad(slice: &[i32]) -> i32 {
    let mut sum = 0;
    for i in 0..slice.len() {
        unsafe { sum += *slice.get_unchecked(i); }
    }
    sum
}

// GOOD: Let the optimizer work
fn sum_good(slice: &[i32]) -> i32 {
    slice.iter().sum()
}
```

Always benchmark the safe version first. Profile to identify bottlenecks. Measure actual improvement from unsafe.

### general-03: Do Not Create Aliases for Types/Methods Named "Unsafe"

**Impact: MEDIUM**

Don't hide unsafe nature behind aliases, re-exports, or wrapper methods. The word "unsafe" signals extra scrutiny.

```rust
// BAD
type SafePointer = *mut u8;  // Still unsafe to dereference!
pub use std::mem::transmute as convert;

// GOOD
type RawHandle = *mut c_void;  // "Raw" signals potential unsafety
pub fn get_value_checked(ptr: *const i32) -> Option<i32> {
    if ptr.is_null() { None }
    else { Some(unsafe { *ptr }) }  // SAFETY: null checked above
}
```

---

## Safety (11 rules)

### safety-01: Be Aware of Memory Safety Issues from Panics

**Impact: CRITICAL** | Clippy: `panic_in_result_fn`

Panics in unsafe code can leave data structures in inconsistent state, leading to UB when caught. Update bookkeeping after operations, not before.

```rust
// BAD: If clone panics after len increment, drop sees uninitialized memory
self.len += 1;
ptr::write(self.ptr.add(self.len - 1), value.clone());

// GOOD: Write first, then increment
ptr::write(self.ptr.add(self.len), value);
self.len += 1;
```

Use panic guards (RAII cleanup) for complex operations:

```rust
struct PanicGuard<'a, T> {
    vec: &'a mut MyVec<T>,
    initialized: usize,
}

impl<T> Drop for PanicGuard<'_, T> {
    fn drop(&mut self) {
        unsafe {
            for i in 0..self.initialized {
                ptr::drop_in_place(self.vec.ptr.add(self.vec.len + i));
            }
        }
    }
}
```

### safety-02: Unsafe Code Authors Must Verify Safety Invariants

**Impact: CRITICAL**

Unsafe blocks transfer responsibility from the compiler to the programmer. Verify: (1) pointer validity, (2) aliasing, (3) initialization, (4) lifetime, (5) type validity, (6) thread safety.

```rust
/// # Safety
///
/// - `ptr` must be non-null and aligned for `Data`
/// - `ptr` must point to `len` consecutive initialized `Data` items
/// - The memory must not be mutated during this call
/// - `len * size_of::<Data>()` must not overflow `isize::MAX`
unsafe fn process(ptr: *const Data, len: usize) {
    debug_assert!(!ptr.is_null());
    debug_assert!(ptr.is_aligned());
    for i in 0..len {
        // SAFETY: Caller guarantees ptr points to len valid items
        let item = &*ptr.add(i);
        process_item(item);
    }
}
```

### safety-03: Do Not Expose Uninitialized Memory in Public APIs

**Impact: CRITICAL** | Clippy: `uninit_assumed_init`

Public APIs must never return or expose uninitialized memory. Use `MaybeUninit<T>` properly, track initialization with length invariants, and only expose initialized portions.

### safety-04: Avoid Double-Free from Panic Safety Issues

**Impact: CRITICAL**

Ensure resources are not freed twice, especially when panics can occur. Patterns: decrement length before reading, use `ManuallyDrop`, use `std::mem::replace`/`swap`, use panic guards.

### safety-05: Consider Safety When Manually Implementing Auto Traits

**Impact: CRITICAL** | Clippy: `non_send_fields_in_send_ty`

Manual `Send`/`Sync` implementations must ensure thread safety. Incorrect implementations cause data races (UB).

**Decision tree**:

- Raw pointers? -> Probably not auto Send/Sync
- Rc/RefCell? -> Not Sync (Rc not Send either)
- Cell/UnsafeCell? -> Not Sync
- Interior mutability? -> Needs synchronization for Sync
- Send: Can another thread safely drop this?
- Sync: Can multiple threads safely call `&self` methods?

```rust
// GOOD: Atomic reference counting with proper bounds
unsafe impl<T: Send + Sync> Send for MyArc<T> {}
unsafe impl<T: Send + Sync> Sync for MyArc<T> {}
```

### safety-06: Do Not Expose Raw Pointers in Public APIs

**Impact: HIGH**

Public APIs should use references, slices, or smart pointers. If raw pointers are necessary, mark the API `unsafe` and document safety requirements.

### safety-07: Provide Unsafe Counterparts for Performance Alongside Safe Methods

**Impact: MEDIUM**

Offer both safe checked and unsafe unchecked versions. Name them `method_name()` (safe) and `method_name_unchecked()` (unsafe). Include `debug_assert!` in the unsafe version.

| Safe Method           | Unsafe Counterpart              |
| --------------------- | ------------------------------- |
| `slice.get(i)`        | `slice.get_unchecked(i)`        |
| `String::from_utf8()` | `String::from_utf8_unchecked()` |

### safety-08: Mutable Return from Immutable Parameter is Wrong

**Impact: CRITICAL** | Clippy: `mut_from_ref`

A function taking `&self` or `&T` must not return `&mut T`. The ONLY valid way is through `UnsafeCell` with guaranteed exclusive access. Use `Cell`, `RefCell`, or `Mutex` for interior mutability.

### safety-09: Add SAFETY Comment Before Any Unsafe Block

**Impact: CRITICAL** | Clippy: `undocumented_unsafe_blocks`

Every `unsafe` block must have a `// SAFETY:` comment explaining: (1) what invariants must hold, (2) why those invariants hold here.

```rust
// SAFETY:
// - buffer.len() >= size_of::<Header>() (asserted above)
// - Header is #[repr(C)] with no padding requirements
unsafe {
    std::ptr::read_unaligned(buffer.as_ptr() as *const Header)
}
```

### safety-10: Add Safety Section in Docs for Public Unsafe Functions

**Impact: HIGH** | Clippy: `missing_safety_doc`

Public `unsafe fn` must have `# Safety` documenting caller obligations: pointer validity, memory state, aliasing, lifetime, thread safety, invariants.

### safety-11: Use assert! Instead of debug_assert! in Unsafe Functions

**Impact: MEDIUM** | Clippy: `debug_assert_with_mut_call`

In unsafe functions, prefer `assert!` for safety invariants since `debug_assert!` is compiled out in release. Use `debug_assert!` only when the invariant is the caller's documented responsibility and performance is critical.

---

## Pointers (6 rules)

### ptr-01: Do Not Share Raw Pointers Across Threads

**Impact: CRITICAL**

Raw pointers are not `Send`/`Sync`. Use `Arc<Mutex<T>>` for shared mutable access or `AtomicPtr` for lock-free pointer sharing.

### ptr-02: Prefer NonNull\<T\> Over *mut T

**Impact: MEDIUM**

`NonNull<T>` enforces non-null at the type level and enables null pointer optimization (`Option<NonNull<T>>` is same size as `*mut T`).

```rust
// NonNull API
NonNull::new(raw_ptr)              // -> Option<NonNull<T>>
unsafe { NonNull::new_unchecked(raw_ptr) }
NonNull::dangling()                // For ZSTs
ptr.as_ptr()                       // -> *mut T
unsafe { ptr.as_ref() }           // -> &T
unsafe { ptr.as_mut() }           // -> &mut T
ptr.cast::<U>()                    // Type cast
```

Use `*mut T` when: null is valid, FFI may return null, or variance matters (NonNull is covariant).

### ptr-03: Use PhantomData\<T\> for Variance and Ownership

**Impact: HIGH**

Use `PhantomData` to tell the compiler about ownership and lifetime relationships for raw pointers.

| Phantom Type             | Meaning           | Variance                        |
| ------------------------ | ----------------- | ------------------------------- |
| `PhantomData<T>`         | Owns T            | Covariant                       |
| `PhantomData<&'a T>`     | Borrows T for 'a  | Covariant in T and 'a           |
| `PhantomData<&'a mut T>` | Mutably borrows T | Invariant in T, covariant in 'a |
| `PhantomData<*mut T>`    | Just has pointer  | Invariant                       |
| `PhantomData<fn(T)>`     | Consumes T        | Contravariant                   |
| `PhantomData<fn() -> T>` | Produces T        | Covariant                       |

### ptr-04: Do Not Dereference Pointers Cast to Misaligned Types

**Impact: HIGH** | Clippy: `cast_ptr_alignment`

Use `read_unaligned` or safe conversion methods (`from_ne_bytes`) instead of dereferencing potentially misaligned pointers.

| Arch    | Misaligned Access |
| ------- | ----------------- |
| x86/x64 | Works but slower  |
| ARM     | UB, may trap      |
| RISC-V  | UB, may trap      |
| WASM    | UB                |

### ptr-05: Do Not Manually Convert Immutable Pointer to Mutable

**Impact: CRITICAL** | Clippy: `cast_ref_to_mut`

Never cast `*const T` to `*mut T` and write through it. This violates aliasing rules and is always UB. Use `UnsafeCell` for interior mutability.

### ptr-06: Prefer pointer::cast Over `as` for Pointer Casting

**Impact: LOW** | Clippy: `ptr_as_ptr`

| Method              | From       | To                          |
| ------------------- | ---------- | --------------------------- |
| `.cast::<U>()`      | `*T`       | `*U`                        |
| `.cast_mut()`       | `*const T` | `*mut T`                    |
| `.cast_const()`     | `*mut T`   | `*const T`                  |
| `.addr()`           | `*T`       | `usize` (nightly)           |
| `.with_addr(usize)` | `*T`       | `*T` (preserves provenance) |

---

## Union (2 rules)

### union-01: Avoid Union Except for C Interop

**Impact: HIGH**

Use `enum` for variant types in Rust. Only use `union` for FFI with `#[repr(C)]` and a tag tracking the active variant.

| Instead of Union     | Use                            |
| -------------------- | ------------------------------ |
| Variant types        | `enum`                         |
| Optional value       | `Option<T>`                    |
| Type punning         | `transmute` or `from_ne_bytes` |
| Uninitialized memory | `MaybeUninit<T>`               |

### union-02: Do Not Use Union Variants Across Different Lifetimes

**Impact: CRITICAL**

Never write to one union field and read from another with a different lifetime. This bypasses lifetime checking and creates dangling references. All reference fields must share the same lifetime parameter.

---

## Memory (6 rules)

### mem-01: Choose Appropriate Data Layout

**Impact: HIGH**

| Attribute              | Use Case                                               |
| ---------------------- | ------------------------------------------------------ |
| `#[repr(C)]`           | C-compatible layout, stable field order                |
| `#[repr(transparent)]` | Single-field struct with same layout as field          |
| `#[repr(packed)]`      | No padding (never create references to packed fields!) |
| `#[repr(align(N))]`    | Minimum alignment of N bytes                           |
| `#[repr(u8)]` etc.     | Enum discriminant type                                 |

Access packed fields via copy or `addr_of!` + `read_unaligned`, never by reference.

### mem-02: Do Not Modify Memory of Other Processes or Dynamic Libraries

**Impact: CRITICAL**

Use proper IPC or FFI mechanisms. Use `libloading` for dynamic libraries.

| Memory Type    | Safe Access              |
| -------------- | ------------------------ |
| Stack/Heap     | Direct / smart pointers  |
| Static         | With synchronization     |
| Shared memory  | Atomic ops, mutexes      |
| Library memory | Through library API      |
| FFI-allocated  | Through C free functions |

### mem-03: Do Not Let String/Vec Auto-Drop Other Allocator's Memory

**Impact: CRITICAL**

Never create `String`, `Vec`, or `Box` from memory allocated outside Rust's allocator. Copy data into Rust-owned types, then free with the correct deallocator.

| Allocator             | Can use Rust Vec/String/Box?     |
| --------------------- | -------------------------------- |
| Rust global allocator | Yes                              |
| C malloc              | No - use wrapper with C free     |
| C++ new               | No - use wrapper with C++ delete |
| Custom allocator      | No - use allocator_api           |
| mmap/shared memory    | No - use munmap                  |

### mem-04: Prefer Reentrant Versions of C-API or Syscalls

**Impact: HIGH**

Use `_r` reentrant versions to avoid data races from global state.

| Non-Reentrant   | Reentrant     | Rust Alternative          |
| --------------- | ------------- | ------------------------- |
| `strtok`        | `strtok_r`    | `str::split`              |
| `localtime`     | `localtime_r` | `chrono` crate            |
| `rand`          | `rand_r`      | `rand` crate              |
| `strerror`      | `strerror_r`  | `std::io::Error`          |
| `getenv`        | None          | `std::env::var`           |
| `gethostbyname` | `getaddrinfo` | `std::net::ToSocketAddrs` |

### mem-05: Use Third-Party Crates for Bitfields

**Impact: MEDIUM**

| Crate              | Use Case                 |
| ------------------ | ------------------------ |
| `bitflags`         | Flag sets (like C enums) |
| `modular-bitfield` | Packed struct fields     |
| `bitvec`           | Arbitrary bit arrays     |
| `packed_struct`    | Binary protocol structs  |
| `deku`             | Binary parsing           |

### mem-06: Use MaybeUninit\<T\> for Uninitialized Memory

**Impact: HIGH** | Clippy: `uninit_assumed_init`, `uninit_vec`

Never use `mem::uninitialized()` (deprecated/UB) or `mem::zeroed()` for types where zero is invalid. Use `MaybeUninit<T>`.

```rust
// MaybeUninit API
MaybeUninit::uninit()           // Uninitialized
MaybeUninit::zeroed()           // Zero-filled
MaybeUninit::new(value)         // Initialized
uninit.write(value)             // Write, returns &mut T
unsafe { uninit.assume_init() } // Extract value
unsafe { uninit.assume_init_ref() }  // Borrow
uninit.as_ptr() / as_mut_ptr()  // Raw pointer access
```

---

## FFI (18 rules)

### ffi-01: Avoid Passing Strings Directly to C

**Impact: HIGH**

Use `CString`/`CStr` at FFI boundaries. Rust strings are not null-terminated.

| Type                | Null-terminated | Encoding | Use        |
| ------------------- | --------------- | -------- | ---------- |
| `String`/`&str`     | No              | UTF-8    | Rust       |
| `CString`/`&CStr`   | Yes             | Byte     | FFI        |
| `OsString`/`&OsStr` | Platform        | Platform | Paths, env |

### ffi-02: Read Documentation Carefully When Using std::ffi Types

**Impact: MEDIUM**

Common pitfalls: `CString::as_ptr()` lifetime (dangling if CString dropped), interior nulls in `CString::new()`, null pointer with `CStr::from_ptr()`.

### ffi-03: Implement Drop for Rust Types Wrapping C Pointers

**Impact: CRITICAL**

Implement `Drop` to call C deallocation. Prevent `Clone`/`Copy` to avoid double-free. Use `NonNull<T>` internally. Encode lifetime dependencies with `PhantomData`.

### ffi-04: Handle Panics When Crossing FFI Boundaries

**Impact: CRITICAL** | Clippy: `panic_in_result_fn`

Wrap FFI callback bodies in `catch_unwind`. Use `extern "C-unwind"` for Rust-to-Rust through C.

### ffi-05: Use Portable Type Aliases

**Impact: HIGH**

Use `std::os::raw` or `libc` crate types. C `long` is 32 bits on Windows, 64 on Unix.

| C Type   | Rust Type     | Notes                     |
| -------- | ------------- | ------------------------- |
| `char`   | `c_char`      | May be signed or unsigned |
| `int`    | `c_int`       | Usually i32               |
| `long`   | `c_long`      | 32 or 64 bits!            |
| `size_t` | `usize`       |                           |
| `void*`  | `*mut c_void` |                           |

### ffi-06: Ensure C-ABI Compatibility for Strings

**Impact: HIGH**

Four ownership patterns: (1) Rust allocates CString, passes ptr, keeps alive; (2) C allocates, Rust borrows via CStr, copies to owned String; (3) C allocates, ownership transferred to Rust wrapper with Drop; (4) Rust allocates via `CString::into_raw()`, C manages.

### ffi-07: Do Not Implement Drop for Types Passed to External Code

**Impact: HIGH**

If external code manages a type's lifetime, don't implement Drop (double-free). Track ownership state if conditional.

| Pattern                  | Who Owns | Rust Drop?    |
| ------------------------ | -------- | ------------- |
| Rust creates, Rust frees | Rust     | Yes           |
| Rust creates, C frees    | C        | No            |
| C creates, C frees       | C        | No (wrapper)  |
| C creates, Rust frees    | Rust     | Yes (wrapper) |

### ffi-08: Handle Errors Properly in FFI

**Impact: HIGH**

Use C-compatible error handling: return codes, errno, out parameters. Rust's `Result`/`Option` don't cross FFI. Use thread-local `LAST_ERROR` for detailed messages.

### ffi-09: Use References Instead of Raw Pointers in Safe Wrappers

**Impact: MEDIUM**

Wrap C functions with safe Rust APIs using references and slices. Use `Option<&T>` for nullable parameters.

### ffi-10: Exported Rust Functions Must Be Thread-Safe

**Impact: CRITICAL**

Functions exported with `#[unsafe(no_mangle)] extern "C"` may be called from multiple threads. Use atomics, `Mutex`, `RwLock`, `OnceLock`, or `thread_local!`.

### ffi-11: Be Careful with #[repr(packed)] Struct Fields

**Impact: HIGH** | Clippy: `unaligned_references`

Never create references to fields in packed structs. Use `addr_of!` + `read_unaligned`, copy by value, or store as byte arrays.

```rust
#[repr(C, packed)]
struct PackedData { header: u8, value: u32 }

impl PackedData {
    fn value(&self) -> u32 {
        unsafe { std::ptr::addr_of!(self.value).read_unaligned() }
    }
}
```

### ffi-12: Document Invariant Assumptions for C Parameters

**Impact: MEDIUM**

Document all assumptions (non-null, alignment, validity, lifetime). Verify at runtime when possible.

### ffi-13: Ensure Consistent Data Layout for Custom Types

**Impact: HIGH**

Use `#[repr(C)]` for all FFI structs. Verify layout at compile time:

```rust
const _: () = {
    assert!(std::mem::size_of::<GoodStruct>() == 12);
    assert!(std::mem::align_of::<GoodStruct>() == 4);
};
```

### ffi-14: Types Used in FFI Should Have Stable Layout

**Impact: HIGH**

Never use `Vec`, `String`, `HashMap`, `Option`, `Box`, or `bool` in FFI signatures. Use raw pointers + lengths, error codes + out parameters, `c_int` or explicit `u8`.

### ffi-15: Validate Non-Robust External Values

**Impact: HIGH**

Validate data from FFI before using as Rust types. Use `TryFrom` instead of `transmute` for enum conversion.

| External Data     | Validation                      |
| ----------------- | ------------------------------- |
| Enum discriminant | Match against valid values      |
| String            | Check UTF-8 or lossy conversion |
| Size/length       | Check against maximum           |
| Pointer           | Check for null                  |
| Boolean           | Explicit 0/1 check              |

### ffi-16: Separate Data and Code for Rust Closures to C

**Impact: HIGH**

Use the trampoline pattern: separate function pointer from closure data via `*mut c_void` user_data.

```
Rust Closure: |x| x * captured_value
     |
     v
+------------------+     +------------------+
| trampoline fn    | --> | closure data     |
| (no captures)    |     | (captured_value) |
+------------------+     +------------------+
     |    user_data ptr        ^
     +-------------------------+
C sees: function pointer + void* user_data
```

### ffi-17: Use Dedicated Opaque Types Instead of c_void

**Impact: MEDIUM**

Create zero-sized marker types to prevent mixing up handle types at compile time.

```rust
#[repr(C)]
pub struct DatabaseHandle {
    _private: [u8; 0],
    _marker: PhantomData<(*mut u8, std::marker::PhantomPinned)>,
}
```

### ffi-18: Avoid Passing Trait Objects to C

**Impact: HIGH**

Trait objects are fat pointers (data + vtable = 16 bytes on 64-bit). C expects thin pointers (8 bytes). Use function pointer + user_data (trampoline pattern) or a C-compatible vtable struct.

---

## I/O (1 rule)

### io-01: Ensure I/O Safety When Using Raw Handles

**Impact: HIGH**

Use Rust 1.63+ I/O safety types instead of `RawFd`.

| Type             | Meaning                                |
| ---------------- | -------------------------------------- |
| `OwnedFd`        | Owns a file descriptor, closes on drop |
| `BorrowedFd<'a>` | Borrows a fd for lifetime 'a           |
| `RawFd`          | Raw integer, no safety guarantees      |
| `AsFd`           | Trait for types that have a fd         |

Windows equivalents: `OwnedHandle`, `BorrowedHandle`, `OwnedSocket`, `BorrowedSocket`.
