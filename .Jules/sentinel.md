## 2025-05-18 - Unbounded HTTP Response Reading (DoS Risk)
**Vulnerability:** The application read the entire response body of the update check URL into memory using `response.text().await` without any size limit.
**Learning:** High-level HTTP clients like `reqwest` often provide convenience methods (`text()`, `bytes()`) that load the full response into memory. For small expected payloads (like configuration files or version checks), this creates a denial-of-service vector if the server returns a massive or infinite stream.
**Prevention:** Always use chunked reading with a size limit (`response.chunk().await` in a loop) when fetching external resources where the size is not strictly controlled or trusted. Enforce timeouts on all network clients.
