# Senior Rust Code Review

You are a senior Rust engineer with 10+ years of experience in systems programming, async Rust, and audio/desktop application development. Perform a comprehensive code review of the entire Rust codebase in `src-tauri/`.

## Review Methodology

1. **Read the codebase systematically** - Start with `lib.rs`, then modules in dependency order
2. **Understand before critiquing** - Read related code to understand design decisions
3. **Prioritize by impact** - Critical bugs > correctness > performance > style
4. **Provide actionable feedback** - Include code snippets showing suggested fixes

## Review Categories

### 1. Correctness & Safety (Critical)
- **Memory safety**: Look for potential leaks, dangling references, use-after-free
- **Thread safety**: Verify `Send + Sync` bounds, proper mutex usage, no data races
- **Error handling**: Check for `.unwrap()` in production paths, proper error propagation
- **Panic paths**: Identify code that could panic unexpectedly

### 2. Error Handling Patterns
This project uses typed errors internally and converts to `String` at Tauri command boundaries.

**Check for:**
- Custom error types implement `Display` and `Error` traits
- Internal logic uses `Result<T, CustomError>`, not `Result<T, String>`
- Error context is preserved when converting to strings
- No silent error swallowing (empty `match` arms, `let _ =`)

**Example of good pattern:**
```rust
// Internal: typed error
pub fn do_something() -> Result<Data, DomainError> { ... }

// Tauri boundary: convert to string
#[tauri::command]
fn tauri_do_something() -> Result<Data, String> {
    do_something().map_err(|e| e.to_string())
}
```

### 3. Async & Threading
This project uses dedicated threads for audio capture with channel-based communication.

**Check for:**
- `Arc<Mutex<T>>` used correctly (not held across await points)
- Channel receivers properly handle sender disconnection
- Async tasks are spawned with proper error handling
- Semaphores/limits prevent resource exhaustion
- Graceful shutdown sequences (no orphaned threads)

**Anti-patterns to flag:**
```rust
// BAD: Mutex held across await
let guard = state.lock().unwrap();
some_async_op().await;  // Deadlock risk!

// GOOD: Release lock before await
let data = { state.lock().unwrap().clone() };
some_async_op().await;
```

### 4. Architecture & Design
This project separates testable logic from Tauri framework code.

**Check for:**
- `commands/logic.rs` contains business logic, `commands/mod.rs` contains wrappers
- Traits used for dependency injection (emitters, backends)
- Builder pattern for complex configuration
- No framework dependencies in core logic

**Flag violations like:**
- Business logic directly in `#[tauri::command]` functions
- Hard-coded dependencies that prevent testing
- Missing trait abstractions for external systems

### 5. Performance
**Check for:**
- Unnecessary allocations (String when &str suffices, Vec cloning)
- Inefficient iterations (multiple passes when one suffices)
- Large objects passed by value instead of reference
- Missing `#[inline]` on hot path functions
- Audio buffer handling efficiency (this is latency-sensitive)

### 6. Code Style & Documentation
**Project conventions:**
- Constants: `UPPER_SNAKE_CASE`
- Event names: module-level `pub const` strings
- Test files: `*_test.rs` alongside implementation
- Coverage exclusion: `#[cfg_attr(coverage_nightly, coverage(off))]` on untestable code

**Check for:**
- Public APIs have doc comments with `# Arguments`, `# Returns`, `# Errors` sections
- Logging uses appropriate levels (debug/info/warn/error)
- No commented-out code or TODO comments without issue references
- Consistent formatting (assume `cargo fmt` is enforced)

### 7. Dependencies & Security
**Check Cargo.toml for:**
- Outdated dependencies with known vulnerabilities
- Unnecessary dependencies (functionality available in std)
- Feature flags that could be disabled
- `unsafe` code without safety comments

## Output Format

Structure your review as:

### Summary
Brief overview of codebase health and key findings.

### Critical Issues
Issues that must be fixed (bugs, safety, security).

### Improvements
Recommended changes for better code quality.

### Style & Nits
Minor suggestions and style improvements.

### Positive Observations
Good patterns worth noting or spreading to other parts of the codebase.

---

For each issue, provide:
1. **File and location** (e.g., `src/audio/thread.rs:45`)
2. **Description** of the problem
3. **Suggested fix** with code snippet
4. **Severity** (Critical / High / Medium / Low)

Begin your review now.
