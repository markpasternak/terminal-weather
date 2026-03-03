## 2025-03-03 - Insecure Default Fallback in reqwest::Client

**Vulnerability:** The HTTP clients (`ForecastClient`, `GeocodeClient`) used a fallback `.unwrap_or_else(|_| Client::new())` if `Client::builder()...build()` failed. This silently dropped critical security configurations like `.timeout()` (leading to DoS via stalled connections) and `.user_agent()`.
**Learning:** In Rust, `reqwest::ClientBuilder::build()` can fail (e.g., native TLS backend errors). Silently swallowing this error with `Client::new()` results in a dangerous, insecure default state where the application runs but lacks required constraints.
**Prevention:** Fail securely. Enforce required configurations by using `.expect("failed to build client")` or by propagating the error, ensuring the application halts rather than operating in an insecure, unprotected mode.
