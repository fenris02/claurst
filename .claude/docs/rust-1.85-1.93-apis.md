# Rust 1.85-1.93 API Reference

Claude's baseline knowledge covers Rust through 1.84. This reference covers features from 1.85 (Feb 2025) through 1.93 (Jan 2026).

## Edition 2024 (Rust 1.85)

| Change                           | Migration                       |
| -------------------------------- | ------------------------------- |
| `unsafe extern "C" {}`           | Add `unsafe` to extern blocks   |
| `#[unsafe(no_mangle)]`           | Wrap unsafe attrs in `unsafe()` |
| `unsafe {}` in unsafe fns        | Explicit unsafe blocks required |
| `static mut` denied              | Use atomics/sync primitives     |
| `gen` reserved                   | Rename identifiers              |
| `set_var`/`remove_var` unsafe    | Wrap in `unsafe {}`             |
| `Future`/`IntoFuture` in prelude | No import needed                |

```rust
// unsafe extern blocks
unsafe extern "C" {
    fn external_func();
}

// unsafe attributes
#[unsafe(no_mangle)]
pub fn exported() {}

// explicit unsafe in unsafe fns
unsafe fn do_stuff(ptr: *const i32) -> i32 {
    unsafe { *ptr }
}

// static mut -> atomics
use std::sync::atomic::{AtomicU32, Ordering};
static COUNTER: AtomicU32 = AtomicU32::new(0);
COUNTER.fetch_add(1, Ordering::SeqCst);

// env functions now unsafe
unsafe { std::env::set_var("KEY", "value"); }
```

**Let chains** (Edition 2024 only):

```rust
if let Some(user) = get_user()
    && let Some(email) = user.email
    && email.contains("@")
{
    send_notification(&email);
}

while let Some(item) = iter.next()
    && item.is_valid()
{
    handle(item);
}
```

## Collections

### `extract_if`

Drain elements matching a predicate, keeping others. Available on Vec (with range), LinkedList, HashMap, HashSet, BTreeMap, BTreeSet.

```rust
let mut vec = vec![1, 2, 3, 4, 5, 6];
let evens: Vec<_> = vec.extract_if(.., |x| *x % 2 == 0).collect();
assert_eq!(evens, vec![2, 4, 6]);
assert_eq!(vec, vec![1, 3, 5]);

// Vec supports range parameter
let extracted: Vec<_> = vec.extract_if(1..4, |x| *x > 2).collect();

// HashMap
let mut map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
let big: HashMap<_, _> = map.extract_if(|_k, v| *v > 1).collect();
```

### `pop_if` / `pop_front_if` / `pop_back_if`

```rust
let mut vec = vec![1, 2, 3, 4];
let popped = vec.pop_if(|x| *x > 3);  // Some(4)
let none = vec.pop_if(|x| *x > 10);   // None

let mut deque = VecDeque::from([1, 2, 3, 4, 5]);
deque.pop_front_if(|x| *x < 2);  // Some(1)
deque.pop_back_if(|x| *x > 4);   // Some(5)
```

### Disjoint Mutable Indexing

```rust
let v = &mut [1, 2, 3, 4, 5];
let [a, b, c] = v.get_disjoint_mut([0, 2, 4]).unwrap();
*a = 10; *b = 30; *c = 50;

let mut map = HashMap::from([("a", 1), ("b", 2), ("c", 3)]);
let [x, y] = map.get_disjoint_mut(["a", "c"]).unwrap();
```

### Tuple Collection

Collect into multiple collections simultaneously (arity 1-12).

```rust
let (evens, odds): (Vec<_>, Vec<_>) =
    (0..10).map(|i| (i * 2, i * 2 + 1)).collect();
```

### Other Collection APIs

```rust
// Cell::update
let counter = Cell::new(0);
counter.update(|x| x + 1);

// Cell::as_array_of_cells
let cell: Cell<[i32; 3]> = Cell::new([1, 2, 3]);
let cells: &[Cell<i32>; 3] = cell.as_array_of_cells();

// btree_map::Entry::insert_entry
let entry = map.entry("key").insert_entry(42);

// core::iter::chain (free function)
let chained: Vec<_> = chain(&[1, 2], &[3, 4]).collect();

// core::array::repeat
let zeros: [i32; 5] = repeat(0);
```

## Integers & Arithmetic

```rust
// midpoint: exact without overflow
0u8.midpoint(255)  // 127
1.0_f64.midpoint(3.0)  // 2.0

// is_multiple_of
15u32.is_multiple_of(5)  // true

// unbounded shifts: return 0 on overflow
1u32.unbounded_shl(40)  // 0, no panic

// cast methods: signed/unsigned reinterpretation
let unsigned: u32 = (-5i32).cast_unsigned();
let back: i32 = unsigned.cast_signed();

// strict arithmetic: panic in ALL builds
200u32.strict_add(50);  // OK: 250
// 200u32.strict_add(u32::MAX);  // Panics in release too!

// carrying/borrowing: extended precision
let (sum, carry) = 250u8.carrying_add(10, false);  // (4, true)
let (low, carry) = a_low.carrying_add(b_low, false);
let (high, _) = a_high.carrying_add(b_high, carry);

let (diff, borrow) = 5u8.borrowing_sub(10, false);  // (251, true)
let (low, high) = 200u8.carrying_mul(200, 0);

// signed-subtraction from unsigned
10u32.checked_sub_signed(-5)  // Some(15)

// NonZero::div_ceil
let n = NonZeroU32::new(10).unwrap();
let d = NonZeroU32::new(3).unwrap();
n.div_ceil(d)  // 4
```

## Async & Concurrency

### Async Closures

```rust
let mut data = vec![];
let closure = async || {
    data.push(fetch_value().await);
};
closure().await;

// New traits: AsyncFn, AsyncFnMut, AsyncFnOnce
async fn call_twice<F: AsyncFnMut()>(f: F) {
    f().await;
    f().await;
}
```

### `OnceLock::wait`

```rust
static CONFIG: OnceLock<Config> = OnceLock::new();
// Thread 1: CONFIG.set(load_config()).unwrap();
// Thread 2: let config = CONFIG.wait();  // Blocks until set
```

### `RwLockWriteGuard::downgrade`

```rust
let mut write_guard = lock.write().unwrap();
*write_guard = 10;
let read_guard = write_guard.downgrade();  // No gap for other writers
```

### `AtomicPtr` Arithmetic

```rust
let ptr = AtomicPtr::new(data.as_mut_ptr());
ptr.fetch_ptr_add(2, Ordering::SeqCst);
ptr.fetch_ptr_sub(1, Ordering::SeqCst);
ptr.fetch_byte_add(4, Ordering::SeqCst);
ptr.fetch_or(0x1, Ordering::SeqCst);   // Pointer tagging
ptr.fetch_and(!0x1, Ordering::SeqCst);
```

### File Locking

```rust
let file = File::open("data.txt")?;
file.lock()?;           // Exclusive (blocks)
file.lock_shared()?;    // Shared (multiple readers)
file.unlock()?;

match file.try_lock() {
    Ok(()) => { /* got lock */ }
    Err(e) if e.kind() == io::ErrorKind::WouldBlock => { /* locked */ }
    Err(e) => return Err(e),
}
```

## Slices & Arrays

### Chunking

```rust
let data = [1, 2, 3, 4, 5, 6, 7, 8];
let (chunks, remainder): (&[[i32; 3]], &[i32]) = data.as_chunks();
// chunks = [[1,2,3], [4,5,6]], remainder = [7,8]

let (remainder, chunks): (&[i32], &[[i32; 3]]) = data.as_rchunks();
// remainder = [1,2], chunks = [[3,4,5], [6,7,8]]
```

### Slice-to-Array Conversion

```rust
let slice = &[1, 2, 3, 4, 5][..];
let arr: Option<&[i32; 5]> = slice.as_array();  // Some
let arr: Option<&[i32; 3]> = slice.as_array();  // None (length mismatch)

if let Some(arr) = slice.as_mut_array::<3>() {
    arr[0] = 10;
}
```

### Split Methods & Character Boundaries

```rust
let (first, rest) = slice.split_off_first().unwrap();
let (last, init) = slice.split_off_last().unwrap();

let s = "Hello, \u{4e16}\u{754c}!";
let ceil = s.ceil_char_boundary(8);   // Start of next char
let floor = s.floor_char_boundary(8); // Start of previous char
let truncated = &s[..s.floor_char_boundary(10)];
```

### Const Operations

```rust
const fn reversed() -> [i32; 5] {
    let mut arr = [1, 2, 3, 4, 5];
    arr.reverse();
    arr
}

const fn rotated() -> [i32; 5] {
    let mut arr = [1, 2, 3, 4, 5];
    arr.rotate_left(2);
    arr  // [3, 4, 5, 1, 2]
}
```

## Strings & Paths

```rust
// String::extend_from_within
let mut s = String::from("Hello");
s.extend_from_within(0..3);  // "HelloHel"

// String/Vec::into_raw_parts
let (ptr, len, capacity) = s.into_raw_parts();
let s = unsafe { String::from_raw_parts(ptr, len, capacity) };

// OsStr::display
let path = OsStr::new("/tmp/file.txt");
println!("Path: {}", path.display());

// Path::file_prefix (removes ALL extensions)
let path = Path::new("/tmp/archive.tar.gz");
path.file_stem()    // "archive.tar" (removes .gz only)
path.file_prefix()  // "archive" (removes all extensions)

// PathBuf::add_extension (keeps existing)
let mut path = PathBuf::from("archive.tar");
path.add_extension("gz");  // "archive.tar.gz"

let backup = PathBuf::from("data.json").with_added_extension("bak");
// "data.json.bak"

// OsString/PathBuf::leak
let leaked: &'static OsStr = OsString::from("hello").leak();
let leaked: &'static Path = PathBuf::from("/tmp").leak();

// CStr/CString cross-type comparison
assert!(c"hello" == CString::new("hello").unwrap());
```

## Pointers & Memory

### Trait Upcasting

```rust
trait Base { fn base_method(&self); }
trait Derived: Base { fn derived_method(&self); }

fn use_base(obj: &dyn Base) { obj.base_method(); }
fn example(obj: &dyn Derived) {
    use_base(obj);  // Automatic upcasting
}

// Works with Box<dyn>, Arc<dyn>, *const dyn
// Useful with Any for downcasting
let any: &dyn Any = obj;  // Upcast
any.downcast_ref::<ConcreteType>();
```

### NonNull Provenance

```rust
let ptr: NonNull<i32> = NonNull::from_ref(&x);
let ptr = NonNull::<i32>::without_provenance(addr);
let addr = ptr.expose_provenance();
let restored = NonNull::<i32>::with_exposed_provenance(addr);
```

### Unsigned Pointer Offset

```rust
let offset: usize = unsafe { p2.offset_from_unsigned(p1) };
```

### MaybeUninit Slice Methods

```rust
let mut uninit: [MaybeUninit<u32>; 4] = MaybeUninit::uninit_array();
uninit.write_copy_of_slice(&source);  // For Copy types
let init: &[u32] = unsafe { uninit.assume_init_ref() };
```

### Zeroed Smart Pointer Constructors

```rust
let boxed: Box<MaybeUninit<[u8; 1024]>> = Box::new_zeroed();
let boxed: Box<[u8; 1024]> = unsafe { boxed.assume_init() };
let slice: Box<[MaybeUninit<u32>]> = Box::new_zeroed_slice(100);
// Same for Rc::new_zeroed(), Arc::new_zeroed()
```

## I/O

### Anonymous Pipes

```rust
let (mut recv, send) = io::pipe()?;
let mut child = Command::new("cargo")
    .arg("build")
    .stdout(send.try_clone()?)
    .stderr(send)
    .spawn()?;
let mut output = String::new();
recv.read_to_string(&mut output)?;
```

### i128/u128 in FFI

```rust
unsafe extern "C" {
    fn process_big_int(value: i128) -> u128;
}
```

### New `io::ErrorKind` Variants

`QuotaExceeded`, `CrossesDevices`

### TCP Quick ACK (Linux)

```rust
use std::os::linux::net::TcpStreamExt;
stream.set_quickack(true)?;
```

## Misc APIs

### Floats

```rust
let next = 1.0_f64.next_up();    // Smallest f64 > 1.0
let prev = 1.0_f64.next_down();  // Largest f64 < 1.0

// Const rounding
const FLOOR: f64 = 3.7_f64.floor();
const CEIL: f64 = 3.2_f64.ceil();
const ROUND_TIES: f64 = 2.5_f64.round_ties_even();  // 2.0
```

### Duration Constructors

```rust
Duration::from_mins(5);
Duration::from_hours(2);
```

### Result::flatten

```rust
let nested: Result<Result<i32, &str>, &str> = Ok(Ok(42));
let flat: Result<i32, &str> = nested.flatten();
```

### fmt::from_fn

```rust
let display = fmt::from_fn(|f| write!(f, "Custom: {}", 42));
fn format_list(items: &[i32]) -> impl fmt::Display + '_ {
    fmt::from_fn(move |f| {
        write!(f, "[")?;
        for (i, item) in items.iter().enumerate() {
            if i > 0 { write!(f, ", ")?; }
            write!(f, "{item}")?;
        }
        write!(f, "]")
    })
}
```

### Const TypeId::of

```rust
const STRING_ID: TypeId = TypeId::of::<String>();
const fn is_string<T: 'static>() -> bool {
    TypeId::of::<T>() == STRING_ID
}
```

### Const Generic Inference with `_`

```rust
let arr: [i32; 5] = [0; _];  // Infers 5
pub fn all_false<const LEN: usize>() -> [bool; LEN] { [false; _] }
```

### `#[diagnostic::do_not_recommend]`

```rust
#[diagnostic::do_not_recommend]
impl<T: Copy> MyTrait for T { }
// Errors won't suggest "implement Copy" when MyTrait is missing
```

### Characters

```rust
const MAX_UTF8: usize = char::MAX_LEN_UTF8;   // 4
const MAX_UTF16: usize = char::MAX_LEN_UTF16; // 2
```

### `use<...>` Precise Capturing

```rust
trait Container {
    fn iter(&self) -> impl Iterator<Item = &u8> + use<'_>;
}
```

## Assembly & SIMD

### Safe `#[target_feature]`

```rust
#[target_feature(enable = "avx2")]
fn simd_operation(data: &[f32]) -> f32 { data.iter().sum() }

fn main() {
    if is_x86_feature_detected!("avx2") {
        unsafe { simd_operation(&data) }
    }
}
```

### Inline Assembly Labels

```rust
unsafe {
    asm!(
        "test {val}, {val}",
        "jz {zero}",
        "mov {val}, 42",
        "jmp {done}",
        val = inout(reg) x,
        zero = label { x = 0; },
        done = label {},
    );
}
```

### `cfg` on `asm!` Lines

```rust
asm!(
    "nop",
    #[cfg(target_feature = "sse2")]
    "movaps xmm0, xmm1",
    #[cfg(target_feature = "avx")]
    "vmovaps ymm0, ymm1",
);
```

### Naked Functions

```rust
#[unsafe(naked)]
pub unsafe extern "sysv64" fn add_one(x: u64) -> u64 {
    naked_asm!("lea rax, [rdi + 1]", "ret");
}
```

## Cargo

- **LLD default** (x86_64 Linux): faster linking. Opt-out: `rustflags = ["-Clinker-features=-lld"]`
- **`cargo publish --workspace`**: publish all crates in dependency order
- **Auto cache cleaning**: 3mo network, 1mo local (no action required)
