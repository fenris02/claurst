# Concurrency & Async

## Choosing an Approach

| Workload                     | Approach          |
| ---------------------------- | ----------------- |
| CPU-bound parallelism        | Threads / `rayon` |
| I/O-bound concurrency        | `async` / `await` |
| Mixed (CPU in async context) | `spawn_blocking`  |

## Shared State

- Prefer message passing (channels) over shared state when possible
- Use `Arc<RwLock<T>>` for read-heavy, `Arc<Mutex<T>>` for write-heavy shared state
- Use `AtomicUsize` / `AtomicBool` for simple counters and flags — not `Mutex` for primitives
- Choose memory ordering carefully: `Relaxed` / `Acquire` / `Release` / `SeqCst`
- Identify lock ordering to prevent deadlocks
- Libraries should avoid `static` and thread-local items when consistent view matters for correctness

## Async Rules

- Never hold a `MutexGuard` across `.await` — scope the lock before awaiting
- Use `tokio::sync::Mutex` instead of `std::sync::Mutex` in async contexts
- Public types should be `Send` for Tokio compatibility; all futures should be `Send`
- Long-running CPU tasks in async should contain `yield_now().await` points (aim for 10-100us between yields)
- Design APIs for batched operations; optimize for throughput (items per CPU cycle)

## Modern APIs (Rust 1.85+)

- **Async closures**: `async || {}` with `AsyncFn`, `AsyncFnMut`, `AsyncFnOnce` traits
- **`OnceLock::wait`**: Block until initialization completes
- **`RwLockWriteGuard::downgrade`**: Write → read lock without releasing
- **`AtomicPtr` arithmetic**: `fetch_ptr_add`, `fetch_ptr_sub`, `fetch_or`, `fetch_and`
- **File locking**: `file.lock()`, `file.lock_shared()`, `file.try_lock()`, `file.unlock()`

## Deprecated Patterns

| Deprecated                         | Use Instead                   |
| ---------------------------------- | ----------------------------- |
| `std::sync::Mutex` (in async)      | `tokio::sync::Mutex`          |
| `std::sync::mpsc`                  | `crossbeam::channel`          |
| `std::sync::Mutex` (perf-critical) | `parking_lot::Mutex`          |
| `lazy_static!`                     | `std::sync::OnceLock` (1.70+) |
| `once_cell::Lazy`                  | `std::sync::LazyLock` (1.80+) |
| `static mut`                       | Atomics or sync primitives    |
