## 2025-03-03 - Insecure Default Fallback in reqwest::Client

**Vulnerability:** The HTTP clients (`ForecastClient`, `GeocodeClient`) used a fallback `.unwrap_or_else(|_| Client::new())` if `Client::builder()...build()` failed. This silently dropped critical security configurations like `.timeout()` (leading to DoS via stalled connections) and `.user_agent()`.
**Learning:** In Rust, `reqwest::ClientBuilder::build()` can fail (e.g., native TLS backend errors). Silently swallowing this error with `Client::new()` results in a dangerous, insecure default state where the application runs but lacks required constraints.
**Prevention:** Fail securely. Enforce required configurations by using `.expect("failed to build client")` or by propagating the error, ensuring the application halts rather than operating in an insecure, unprotected mode.

## 2025-03-03 - Unbounded JSON Deserialization DoS

**Vulnerability:** The application used unbounded `response.json().await` when fetching payloads from external APIs (`forecast`, `geocode`, `geoip`). A compromised or malicious server returning an extremely large JSON payload could cause the application to exhaust available memory and crash (Denial of Service).
**Learning:** `reqwest`'s default `.json()` method buffers the entire response into memory before deserializing. Even when communicating with trusted third-party APIs, network boundaries should be considered untrusted, and resource limits must be enforced defensively.
**Prevention:** To prevent DoS via memory exhaustion, avoid unbounded `.json().await`. Instead, use a chunked reading loop with `response.chunk().await`, enforce a strict maximum byte limit (e.g., 2MB) by accumulating chunks manually, and then deserialize the bounded buffer using `serde_json::from_slice`.
## 2025-03-03 - Enforced Security Configuration across all HTTP Clients

**Vulnerability:** A previous entry noted that falling back to an unprotected `reqwest::Client` without a `.timeout()` or `.user_agent()` is dangerous. However, `GeoIpClient` used an `ok()` fallback, meaning if the underlying TLS implementation failed to initialize, the client would silently refuse to build instead of properly failing securely and halting the program.
**Learning:** `reqwest::ClientBuilder::build()` can fail due to critical system-level issues (like missing TLS certificates). Failing silently masks these critical infrastructure issues.
**Prevention:** Fail securely in all locations where HTTP clients are constructed. Enforced this by ensuring `geoip.rs` now explicitly expects successful creation (just like `forecast.rs` and `geocode.rs`).

## 2025-03-03 - Unbounded Local Config File Read DoS

**Vulnerability:** The settings loader used `fs::read_to_string` to load `settings.json`. If a malicious user on a shared system replaced the file with an enormous payload or a symlink to `/dev/zero`, the application would attempt to read it entirely into memory, resulting in an Out-Of-Memory (OOM) crash (Denial of Service).
**Learning:** Even when reading local configuration files from user-controlled paths (like `~/.config`), defensive programming requires bounding read sizes to prevent resource exhaustion attacks via the filesystem.
**Prevention:** Replace unbounded `fs::read_to_string` with an explicitly bounded read using `std::fs::File`, `std::io::Read::read_to_string`, and `.take(max_bytes)` to enforce a strict memory limit.

## 2026-03-10 - Panic DoS via Unbounded refresh_interval_secs

**Vulnerability:** The application accepted unbounded `u64` values for `refresh_interval_secs` via CLI arguments and `settings.json`. If a maliciously large value (e.g. `u64::MAX`) was provided, it would trigger an out-of-bounds panic inside `tokio::time::sleep` or `Duration::from_secs_f32`, immediately crashing the process (Denial of Service).
**Learning:** All configurations that interact with underlying runtime components (like Tokio timers) must be securely clamped to reasonable bounds, even if the data originates from seemingly trusted local config files or user input.
**Prevention:** Clamped `refresh_interval_secs` to a safe maximum bound (24 hours) during CLI parsing and settings deserialization to prevent application panics.

## 2025-03-03 - Enforced Security Configuration across all HTTP Clients

**Vulnerability:** A previous entry noted that falling back to an unprotected `reqwest::Client` without a `.timeout()` or `.user_agent()` is dangerous. However, `ForecastClient`, `GeocodeClient`, and `GeoIpClient` used `.expect()` on builder creation, meaning if the underlying TLS implementation failed to initialize, the application would encounter a hard panic (Denial of Service) instead of properly failing securely and halting the component/program via predictable channels.
**Learning:** `reqwest::ClientBuilder::build()` can fail due to critical system-level issues (like missing TLS certificates). While `.expect()` is better than `.ok()`, failing via a hard panic inside UI-triggered async operations drops the application.
**Prevention:** Fail securely in all locations where HTTP clients are constructed. Enforced this by updating all client builders to return `Result<Self>` (or `Option<Self>`) and allowing the failure to bubble up into the event loop (e.g., `AppEvent::FetchFailed`) to cleanly alert the user without crashing the TUI.
