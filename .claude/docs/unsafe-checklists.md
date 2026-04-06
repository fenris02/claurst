# Unsafe Code Checklists

## Checklist: Before Writing Unsafe Code

### 1. Do You Really Need Unsafe?

- [ ] Have you tried all safe alternatives?
- [ ] Can you restructure the code to satisfy the borrow checker?
- [ ] Would interior mutability (`Cell`, `RefCell`, `Mutex`) solve the problem?
- [ ] Is there a safe crate that already does this?
- [ ] Is the performance gain (if any) worth the safety risk?

**If you answered "no" to all, proceed with unsafe.**

### 2. What Unsafe Operation Do You Need?

Identify which specific unsafe operation you're performing:

- [ ] Dereferencing a raw pointer (`*const T`, `*mut T`)
- [ ] Calling an `unsafe` function
- [ ] Accessing a mutable static variable
- [ ] Implementing an unsafe trait (`Send`, `Sync`, etc.)
- [ ] Accessing fields of a `union`
- [ ] Using `extern "C"` functions (FFI)

### 3. Safety Invariants

For each unsafe operation, document the invariants:

**For Pointer Dereference:**

- [ ] Is the pointer non-null?
- [ ] Is the pointer properly aligned for the type?
- [ ] Does the pointer point to valid, initialized memory?
- [ ] Is the memory not being mutated by other code?
- [ ] Will the memory remain valid for the entire duration of use?

**For Mutable Aliasing:**

- [ ] Are you creating multiple mutable references to the same memory?
- [ ] Is there any possibility of aliasing `&mut` and `&`?
- [ ] Have you verified no other code can access this memory?

**For FFI:**

- [ ] Is the function signature correct (types, ABI)?
- [ ] Are you handling potential null pointers?
- [ ] Are you handling potential panics (catch_unwind)?
- [ ] Is memory ownership clear (who allocates, who frees)?

**For Send/Sync:**

- [ ] Is concurrent access properly synchronized?
- [ ] Are there any data races possible?
- [ ] Does the type truly satisfy the trait requirements?

### 4. Panic Safety

- [ ] What happens if this code panics at any line?
- [ ] Are data structures left in a valid state on panic?
- [ ] Do you need a panic guard for cleanup?
- [ ] Could a destructor see invalid state?

### 5. Documentation

- [ ] Have you written a `// SAFETY:` comment explaining:
  - What invariants must hold?
  - Why those invariants are upheld here?

- [ ] For `unsafe fn`, have you written `# Safety` docs explaining:
  - What the caller must guarantee?
  - What happens if requirements are violated?

### 6. Testing and Verification

- [ ] Can you add debug assertions to verify invariants?
- [ ] Have you tested with Miri (`cargo miri test`)?
- [ ] Have you tested with address sanitizer (`RUSTFLAGS="-Zsanitizer=address"`)?
- [ ] Have you considered fuzzing the unsafe code?

### Common SAFETY Comments

```rust
// SAFETY: We checked that index < len above, so this is in bounds.

// SAFETY: The pointer was created from a valid reference and hasn't been invalidated.

// SAFETY: We hold the lock, guaranteeing exclusive access.

// SAFETY: The type is #[repr(C)] and all fields are initialized.

// SAFETY: Caller guarantees the pointer is non-null and properly aligned.
```

### Decision Flowchart

```
Need unsafe?
     |
     v
Can you use safe Rust? --Yes--> Don't use unsafe
     |
     No
     v
Can you use existing safe abstraction? --Yes--> Use it (std, crates)
     |
     No
     v
Document all invariants
     |
     v
Add SAFETY comments
     |
     v
Write the unsafe code
     |
     v
Test with Miri
```

---

## Checklist: Reviewing Unsafe Code

### 1. Surface-Level Checks

- [ ] Does every `unsafe` block have a `// SAFETY:` comment?
- [ ] Does every `unsafe fn` have `# Safety` documentation?
- [ ] Are the safety comments specific and verifiable, not vague?
- [ ] Is the unsafe code minimized (smallest possible unsafe block)?

### 2. Pointer Validity

For each pointer dereference:

- [ ] **Non-null**: Is null checked before dereference?
- [ ] **Aligned**: Is alignment verified or guaranteed by construction?
- [ ] **Valid**: Does the pointer point to allocated memory?
- [ ] **Initialized**: Is the memory initialized before reading?
- [ ] **Lifetime**: Is the memory valid for the entire use duration?
- [ ] **Unique**: For `&mut`, is there only one mutable reference?

### 3. Memory Safety

- [ ] **No aliasing**: Are `&` and `&mut` never created to the same memory simultaneously?
- [ ] **No use-after-free**: Is memory not accessed after deallocation?
- [ ] **No double-free**: Is memory freed exactly once?
- [ ] **No data races**: Is concurrent access properly synchronized?
- [ ] **Bounds checked**: Are array/slice accesses in bounds?

### 4. Type Safety

- [ ] **Transmute**: Are transmuted types actually compatible?
- [ ] **Repr**: Do FFI types have `#[repr(C)]`?
- [ ] **Enum values**: Are enum discriminants validated from external sources?
- [ ] **Unions**: Is the correct union field accessed?

### 5. Panic Safety

- [ ] What state is the program in if this code panics?
- [ ] Are partially constructed objects properly cleaned up?
- [ ] Do Drop implementations see valid state?
- [ ] Is there a panic guard if needed?

### 6. FFI-Specific Checks

- [ ] **Types**: Do Rust types match C types exactly?
- [ ] **Strings**: Are strings properly null-terminated?
- [ ] **Ownership**: Is it clear who owns/frees memory?
- [ ] **Thread safety**: Are callbacks thread-safe?
- [ ] **Panic boundary**: Are panics caught before crossing FFI?
- [ ] **Error handling**: Are C-style errors properly handled?

### 7. Concurrency Checks

- [ ] **Send/Sync**: Are manual implementations actually sound?
- [ ] **Atomics**: Are memory orderings correct?
- [ ] **Locks**: Is there potential for deadlock?
- [ ] **Data races**: Is all shared mutable state synchronized?

### 8. Red Flags (Require Extra Scrutiny)

| Pattern              | Concern                        |
| -------------------- | ------------------------------ |
| `transmute`          | Type compatibility, provenance |
| `as` on pointers     | Alignment, type punning        |
| `static mut`         | Data races                     |
| `*const T as *mut T` | Aliasing violation             |
| Manual `Send`/`Sync` | Thread safety                  |
| `assume_init`        | Initialization                 |
| `set_len` on Vec     | Uninitialized memory           |
| `from_raw_parts`     | Lifetime, validity             |
| `offset`/`add`/`sub` | Out of bounds                  |
| FFI callbacks        | Panic safety                   |

### 9. Verification Questions

Ask the author:

- "What would happen if [X invariant] was violated?"
- "How do you know [pointer/reference] is valid here?"
- "What if this panics at [specific line]?"
- "Who is responsible for freeing this memory?"

### 10. Testing Requirements

- [ ] Has this been tested with Miri?
- [ ] Are there unit tests covering edge cases?
- [ ] Are there tests for error conditions?
- [ ] Has concurrent code been tested under stress?

### Review Severity Guide

| Severity             | Requires                                 |
| -------------------- | ---------------------------------------- |
| `transmute`          | Two reviewers, Miri test                 |
| Manual `Send`/`Sync` | Thread safety expert review              |
| FFI                  | Documentation of C interface             |
| `static mut`         | Justification for not using atomic/mutex |
| Pointer arithmetic   | Bounds proof                             |

---

## Common Unsafe Pitfalls and Fixes

### Pitfall 1: Dangling Pointer from Local

```rust
// BAD
fn bad() -> *const i32 {
    let x = 42;
    &x as *const i32  // Dangling after return!
}

// GOOD
fn good() -> Box<i32> {
    Box::new(42)  // Heap allocation lives beyond function
}
```

### Pitfall 2: CString Lifetime

```rust
// BAD
fn bad() -> *const c_char {
    let s = CString::new("hello").unwrap();
    s.as_ptr()  // Dangling! CString dropped
}

// GOOD: Caller keeps CString alive
fn good(s: &CString) -> *const c_char {
    s.as_ptr()
}

// Or transfer ownership (caller must free with CString::from_raw)
fn also_good(s: CString) -> *const c_char {
    s.into_raw()
}
```

### Pitfall 3: Vec set_len with Uninitialized Data

```rust
// BAD
fn bad() -> Vec<String> {
    let mut v = Vec::with_capacity(10);
    unsafe { v.set_len(10); }  // Strings are uninitialized!
    v
}

// GOOD
fn good() -> Vec<String> {
    let mut v = Vec::new();
    v.resize(10, String::new());
    v
}
```

### Pitfall 4: Reference to Packed Field

```rust
#[repr(packed)]
struct Packed { a: u8, b: u32 }

// BAD
fn bad(p: &Packed) -> &u32 {
    &p.b  // UB: misaligned reference!
}

// GOOD
fn good(p: &Packed) -> u32 {
    unsafe { std::ptr::addr_of!(p.b).read_unaligned() }
}
```

### Pitfall 5: Mutable Aliasing Through Raw Pointers

```rust
// BAD
fn bad() {
    let mut x = 42;
    let ptr1 = &mut x as *mut i32;
    let ptr2 = &mut x as *mut i32;  // Already have ptr1!
    unsafe {
        *ptr1 = 1;
        *ptr2 = 2;  // Aliasing mutable pointers!
    }
}

// GOOD: Single pointer, sequential access
fn good() {
    let mut x = 42;
    let ptr = &mut x as *mut i32;
    unsafe {
        *ptr = 1;
        *ptr = 2;
    }
}
```

### Pitfall 6: Transmute to Wrong Size

```rust
// BAD
fn bad() {
    let x: u32 = 42;
    let y: u64 = unsafe { std::mem::transmute(x) };  // UB: size mismatch!
}

// GOOD
fn good() {
    let x: u32 = 42;
    let y: u64 = x as u64;  // Use conversion
}
```

### Pitfall 7: Invalid Enum Discriminant

```rust
#[repr(u8)]
enum Status { A = 0, B = 1, C = 2 }

// BAD
fn bad(raw: u8) -> Status {
    unsafe { std::mem::transmute(raw) }  // UB if raw > 2!
}

// GOOD
fn good(raw: u8) -> Option<Status> {
    match raw {
        0 => Some(Status::A),
        1 => Some(Status::B),
        2 => Some(Status::C),
        _ => None,
    }
}
```

### Pitfall 8: FFI Panic Unwinding

```rust
// BAD: panic unwinds across FFI boundary
#[unsafe(no_mangle)]
extern "C" fn callback(x: i32) -> i32 {
    if x < 0 {
        panic!("negative!");  // UB: unwinding across FFI!
    }
    x * 2
}

// GOOD
#[unsafe(no_mangle)]
extern "C" fn callback(x: i32) -> i32 {
    std::panic::catch_unwind(|| {
        if x < 0 { panic!("negative!"); }
        x * 2
    }).unwrap_or(-1)
}
```

### Pitfall 9: Double Free from Clone + into_raw

```rust
// BAD: Both clones "own" same pointer, both Drop = double free
struct Handle(*mut c_void);
impl Clone for Handle {
    fn clone(&self) -> Self { Handle(self.0) }
}
impl Drop for Handle {
    fn drop(&mut self) { unsafe { free(self.0); } }
}

// GOOD: Don't implement Clone for owning handles
struct Handle(*mut c_void);
// No Clone impl — single owner
```

### Pitfall 10: Forget Doesn't Run Destructors

```rust
// BAD
fn bad() {
    let guard = lock.lock();
    std::mem::forget(guard);  // Lock never released!
}

// GOOD: Let guard drop naturally, or explicit drop(guard)
```

### Quick Detection Reference

| Pitfall            | Detection     | Fix                              |
| ------------------ | ------------- | -------------------------------- |
| Dangling pointer   | Miri          | Extend lifetime or heap allocate |
| Uninitialized read | Miri          | Use MaybeUninit properly         |
| Misaligned access  | Miri, UBsan   | read_unaligned, copy by value    |
| Data race          | TSan          | Use atomics or mutex             |
| Double free        | ASan          | Track ownership carefully        |
| Invalid enum       | Manual review | Use TryFrom                      |
| FFI panic          | Testing       | catch_unwind                     |
| Type confusion     | Miri          | Match types exactly              |
