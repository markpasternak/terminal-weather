## 2025-03-03 - Insecure Default Fallback in reqwest::Client

**Vulnerability:** The HTTP clients (`ForecastClient`, `GeocodeClient`) used a fallback `.unwrap_or_else(|_| Client::new())` if `Client::builder()...build()` failed. This silently dropped critical security configurations like `.timeout()` (leading to DoS via stalled connections) and `.user_agent()`.
**Learning:** In Rust, `reqwest::ClientBuilder::build()` can fail (e.g., native TLS backend errors). Silently swallowing this error with `Client::new()` results in a dangerous, insecure default state where the application runs but lacks required constraints.
**Prevention:** Fail securely. Enforce required configurations by using `.expect("failed to build client")` or by propagating the error, ensuring the application halts rather than operating in an insecure, unprotected mode.

## 2025-03-03 - Unbounded JSON Deserialization DoS

**Vulnerability:** The application used unbounded `response.json().await` when fetching payloads from external APIs (`forecast`, `geocode`, `geoip`). A compromised or malicious server returning an extremely large JSON payload could cause the application to exhaust available memory and crash (Denial of Service).
**Learning:** `reqwest`'s default `.json()` method buffers the entire response into memory before deserializing. Even when communicating with trusted third-party APIs, network boundaries should be considered untrusted, and resource limits must be enforced defensively.
**Prevention:** To prevent DoS via memory exhaustion, avoid unbounded `.json().await`. Instead, use a chunked reading loop with `response.chunk().await`, enforce a strict maximum byte limit (e.g., 2MB) by accumulating chunks manually, and then deserialize the bounded buffer using `serde_json::from_slice`.
