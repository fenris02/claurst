# Serde Patterns

## Derive Basics

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserProfile {
    pub user_id: u64,
    pub display_name: String,
    pub email: Option<String>,
}
```

- Always derive both `Serialize` and `Deserialize` unless you have a reason not to
- Use `#[serde(rename_all = "camelCase")]` for JSON APIs, `#[serde(rename_all = "snake_case")]` for config files
- Place `serde` derives after `Debug`, `Clone`, `PartialEq` in the derive list

## Common Attributes

### Struct-Level

| Attribute                                  | Purpose                                                                     |
| ------------------------------------------ | --------------------------------------------------------------------------- |
| `#[serde(rename_all = "...")]`             | Rename all fields (camelCase, snake_case, SCREAMING_SNAKE_CASE, kebab-case) |
| `#[serde(deny_unknown_fields)]`            | Reject input with extra fields — use for config files and strict APIs       |
| `#[serde(default)]`                        | Use `Default::default()` for all missing fields                             |
| `#[serde(tag = "type")]`                   | Internally tagged enum: `{"type": "variant", ...}`                          |
| `#[serde(tag = "type", content = "data")]` | Adjacently tagged enum: `{"type": "variant", "data": ...}`                  |
| `#[serde(untagged)]`                       | Untagged enum — tries each variant in order (avoid when possible)           |

### Field-Level

| Attribute                                           | Purpose                                             |
| --------------------------------------------------- | --------------------------------------------------- |
| `#[serde(default)]`                                 | Use field type's `Default` if missing               |
| `#[serde(default = "path")]`                        | Use custom function for default value               |
| `#[serde(skip)]`                                    | Exclude from both serialization and deserialization |
| `#[serde(skip_serializing_if = "Option::is_none")]` | Omit `None` values from output                      |
| `#[serde(rename = "name")]`                         | Rename a specific field                             |
| `#[serde(flatten)]`                                 | Inline nested struct fields                         |
| `#[serde(with = "module")]`                         | Custom serialize/deserialize via module             |
| `#[serde(deserialize_with = "path")]`               | Custom deserializer for one field                   |

## Enum Representations

Choose the right tagging strategy:

```rust
// Externally tagged (default) — {"variant_name": {...}}
#[derive(Serialize, Deserialize)]
enum Message {
    Text(String),
    Image { url: String, width: u32 },
}

// Internally tagged — {"type": "text", "content": "..."}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Event {
    UserCreated { user_id: u64 },
    UserDeleted { user_id: u64 },
}

// Adjacently tagged — {"type": "text", "data": "..."}
#[derive(Serialize, Deserialize)]
#[serde(tag = "type", content = "data", rename_all = "snake_case")]
enum Response {
    Success(Data),
    Error(String),
}
```

- Prefer internally tagged for APIs — it's the most common JSON convention
- Avoid `#[serde(untagged)]` — error messages are poor, and order-dependent matching is fragile
- Always add `rename_all` to tagged enums for consistent serialized names

## Strict Deserialization

Use `deny_unknown_fields` for configuration and APIs where unexpected fields indicate a mistake:

```rust
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Config {
    pub listen_addr: String,
    pub max_connections: u32,
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    30
}
```

## Newtype Wrappers

Serde works with newtype wrappers transparently:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct UserId(pub u64);

// Serializes as a bare number: 42
// No need for #[serde(transparent)] on single-field tuple structs
```

Use `#[serde(transparent)]` on named-field newtypes:

```rust
#[derive(Debug, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Email {
    value: String,
}
```

## Custom Serialization

For types that need special handling, use `#[serde(with = "module")]`:

```rust
mod duration_secs {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(d.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}

#[derive(Serialize, Deserialize)]
struct Task {
    name: String,
    #[serde(with = "duration_secs")]
    timeout: Duration,
}
```

Common crates for custom serialization:

| Crate             | Purpose                                                             |
| ----------------- | ------------------------------------------------------------------- |
| `serde_with`      | Attribute macros for common patterns (display, duration, hex, etc.) |
| `time`            | `time::serde` module for date/time formatting                       |
| `chrono`          | `chrono::serde` for DateTime serialization                          |
| `humantime-serde` | Human-readable durations ("5m 30s")                                 |

## Anti-Patterns

| Anti-Pattern                            | Use Instead                                 |
| --------------------------------------- | ------------------------------------------- |
| `serde_json::Value` for known structure | Define a typed struct                       |
| Manual `Serialize`/`Deserialize` impls  | `#[serde(with)]` or `serde_with` crate      |
| `#[serde(untagged)]` for polymorphism   | Internally tagged enum with `#[serde(tag)]` |
| String fields for structured data       | Newtypes or parsed types                    |
| `#[serde(default)]` on every field      | `#[serde(default)]` on the struct           |
| Ignoring unknown fields in config       | `#[serde(deny_unknown_fields)]`             |

## Format Crates

| Format            | Crate                                          |
| ----------------- | ---------------------------------------------- |
| JSON              | `serde_json`                                   |
| TOML              | `toml`                                         |
| YAML              | `serde_yaml` (maintenance mode) or `serde_yml` |
| MessagePack       | `rmp-serde`                                    |
| CBOR              | `ciborium`                                     |
| Binary            | `bincode` or `postcard`                        |
| CSV               | `csv`                                          |
| URL query strings | `serde_urlencoded` or `serde_qs`               |
