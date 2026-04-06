# Ownership & Memory

## Borrowing & Lifetimes

- Borrow (`&` / `&mut`) by default; only clone when genuinely needed
- Don't clone to satisfy the borrow checker — restructure code instead
- Copy values out before mutating the container (e.g., `let first = data[0];` before `data.push(4);`)
- Iterate by reference (`for item in &vec`) unless you need ownership
- Use `Cow<'_, T>` when data is sometimes borrowed, sometimes owned
- Use meaningful lifetime names (`'src` not `'a`) when elision doesn't apply
- Shadow variables through transformations (`let input = input.trim();`) — no `raw_x`/`parsed_x` prefixes

## Smart Pointers

| Type                          | Use When                                                |
| ----------------------------- | ------------------------------------------------------- |
| `Box<T>`                      | Heap allocation, recursive types, single owner          |
| `Rc<T>`                       | Single-thread shared ownership                          |
| `Arc<T>`                      | Multi-thread shared ownership                           |
| `Cell<T>`                     | Interior mutability for Copy types (single-thread)      |
| `RefCell<T>`                  | Interior mutability with runtime checks (single-thread) |
| `OnceLock<T>` / `LazyLock<T>` | Thread-safe lazy initialization                         |

- Don't use `Arc` when `Rc` suffices
- Don't use `Box` for small types
- Don't use `RefCell` everywhere — design clear ownership instead
- `Rc` when single owner → just use the value directly

## Iterator & Collection Patterns

- Chain iterators instead of collecting then processing
- Use iterator adaptors (`map`, `filter`, `flat_map`) over index-based loops
- Prefer `for item in &vec` over `for i in 0..vec.len()` with `vec[i]`
- Use `push_str` or `+` for simple string concatenation instead of `format!`
- Use `s.bytes()` over `s.chars()` when working with ASCII

## Optimization

- Write efficient code by default — correct algorithm, appropriate data structures, no unnecessary allocations
- Profile before micro-optimizing; measure after
- For applications, consider `mimalloc` as global allocator (up to 25% gains on allocation-heavy paths)
- Identify hot paths early; benchmark with `criterion` or `divan`
- Common perf issues: frequent re-allocations, cloned strings/collections, redundant hashing, default hasher where collision resistance isn't needed
