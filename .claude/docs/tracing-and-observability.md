# Tracing & Observability

## Core Crates

| Crate                   | Purpose                                           |
| ----------------------- | ------------------------------------------------- |
| `tracing`               | Structured, span-based instrumentation API        |
| `tracing-subscriber`    | Subscriber layers (formatting, filtering, output) |
| `tracing-appender`      | Non-blocking file and rolling log output          |
| `tracing-opentelemetry` | OpenTelemetry integration                         |

Never use `println!`, `eprintln!`, or the `log` crate directly. Use `tracing` macros exclusively.

## Log Levels

| Level    | Use For                                                         |
| -------- | --------------------------------------------------------------- |
| `error!` | Unrecoverable failures requiring operator attention             |
| `warn!`  | Degraded behavior, recoverable failures, approaching limits     |
| `info!`  | Significant state changes, request completion, startup/shutdown |
| `debug!` | Detailed control flow, intermediate values for debugging        |
| `trace!` | Very fine-grained detail, per-iteration logging                 |

- Default production level: `info`
- Default development level: `debug`
- Use `trace!` sparingly — it generates high volume

## Structured Logging

Use message templates with named fields, not string interpolation:

```rust
// GOOD: structured fields
tracing::info!(
    user_id = %user.id,
    email = %user.email,
    action = "login",
    "user.auth.success"
);

// BAD: interpolated string
tracing::info!("User {} logged in with email {}", user.id, user.email);
```

- Use `%` for Display formatting, `?` for Debug formatting
- Name events with hierarchical dot-notation: `<component>.<operation>.<state>`
- Keep field names consistent across the codebase
- Follow OpenTelemetry semantic conventions for common attributes

## Spans

Spans represent a unit of work with a beginning and end. Use `#[instrument]` for function-level spans:

```rust
#[tracing::instrument(skip(password), fields(user_id = %user.id))]
async fn authenticate(user: &User, password: &str) -> Result<Token> {
    // All log events within this function are associated with this span
    tracing::debug!("checking credentials");
    // ...
}
```

- `skip(field)` — omit sensitive or large fields from the span
- `fields(key = value)` — add computed fields to the span
- `level = "debug"` — set span level (default is `info`)
- `name = "custom_name"` — override the default function name
- `err` — automatically record `Err` results as span events

Manual spans for non-function scopes:

```rust
let span = tracing::info_span!("db.query", table = "users", query_id = %id);
let _guard = span.enter();
// work happens here, span closes when _guard drops
```

In async code, use `.instrument(span)` instead of `span.enter()`:

```rust
async fn process(items: Vec<Item>) {
    let span = tracing::info_span!("batch.process", count = items.len());
    do_work(items).instrument(span).await;
}
```

## Subscriber Setup

Configure the subscriber in `main()`, not in library code. Libraries should only use `tracing` macros.

```rust
use tracing_subscriber::{fmt, EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| EnvFilter::new("info")))
        .with(fmt::layer()
            .with_target(true)
            .with_thread_ids(false)
            .with_file(true)
            .with_line_number(true))
        .init();
}
```

- Use `EnvFilter` to control log levels via `RUST_LOG` environment variable
- `RUST_LOG=my_crate=debug,tower_http=info` — per-crate filtering
- Use `fmt::layer().json()` for machine-readable output in production
- Use `fmt::layer().pretty()` for human-readable output in development

## Sensitive Data

Never log secrets, tokens, passwords, or PII:

```rust
// Wrapper that redacts on Display/Debug
pub struct Redacted<T>(pub T);

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[REDACTED]")
    }
}

tracing::info!(token = %Redacted(token), "auth.token.issued");
```

- Use `skip` in `#[instrument]` for sensitive parameters
- Implement custom `Debug` for types containing sensitive data
- Review log output in CI to catch accidental leaks

## OpenTelemetry Integration

For distributed tracing, add `tracing-opentelemetry`:

```rust
use tracing_opentelemetry::OpenTelemetryLayer;

tracing_subscriber::registry()
    .with(OpenTelemetryLayer::new(tracer))
    .with(fmt::layer())
    .init();
```

- Spans become OpenTelemetry spans automatically
- Use `tracing::Span::current().context()` to propagate context across service boundaries
- Export to Jaeger, Zipkin, or any OTLP-compatible backend

## Common Patterns

| Pattern             | Implementation                                                             |
| ------------------- | -------------------------------------------------------------------------- |
| Request ID          | Add `request_id` field to the root span per request                        |
| Timing              | Spans automatically track duration — no manual timing needed               |
| Error context       | Use `.inspect_err(\|e\| tracing::error!(error = %e, "operation.failed"))`  |
| Conditional logging | Use `tracing::enabled!(Level::DEBUG)` to guard expensive formatting        |
| Test output         | Use `tracing-test` crate or `tracing_subscriber::fmt().with_test_writer()` |
