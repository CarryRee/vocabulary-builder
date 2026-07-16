# AGENTS.md

## Project Overview

This is a desktop application built with:

* Tauri 2.x
* Rust backend
* Dioxus frontend
* Cargo workspace

The application should prioritize:

* Rust safety
* Clean architecture
* Minimal dependencies
* Cross-platform compatibility

---

# Technology Stack

## Backend

* Language: Rust
* Framework: Tauri 2.x
* Build system: Cargo

Main directory:

```
src-tauri/
```

Rules:

* Use Rust stable version
* Run `cargo fmt` before committing
* Run `cargo clippy` to check code quality
* Prefer `Result<T, E>` error handling
* Avoid `unwrap()` and `expect()` in production code
* Use async APIs where appropriate

---

## Frontend

* Framework: Dioxus
* Language: Rust

Main directory:

```
src/
```

Rules:

* Use Dioxus components
* Keep UI components small and reusable
* Prefer signals/state management provided by Dioxus
* Avoid unnecessary global state
* Keep business logic separate from UI components

Example structure:

```
src/
├── components/
│   └── reusable UI components
├── pages/
│   └── application pages
├── services/
│   └── backend communication
├── models/
│   └── data structures
└── main.rs
```

---

# Tauri Communication Rules

Frontend and backend communication must use Tauri commands.

Rules:

* Define commands in `src-tauri`
* Keep command parameters simple
* Use serializable structs for data exchange
* Avoid exposing internal backend implementation

Example:

```rust
#[tauri::command]
async fn get_data() -> Result<Data, String> {
    Ok(data)
}
```

---

# Rust Coding Style

Follow Rust official style:

* Use `cargo fmt`
* Use meaningful variable names
* Add comments for complex logic
* Prefer ownership over unnecessary cloning
* Avoid unsafe code unless required

Error handling:

Preferred:

```rust
fn process() -> Result<String, Error>
```

Avoid:

```rust
fn process() -> String {
    value.unwrap()
}
```

---

# Dependency Rules

Before adding a dependency:

1. Check whether Rust standard library can solve it
2. Prefer mature and maintained crates
3. Avoid duplicate dependencies

Update:

```
Cargo.toml
```

carefully.

---

# File Modification Rules

Before modifying code:

1. Understand existing architecture
2. Avoid unnecessary refactoring
3. Keep changes focused
4. Do not modify generated files

Generated directories:

```
target/
dist/
```

should not be edited manually.

---

# Testing Requirements

Before submitting changes:

Backend:

```bash
cargo test
cargo clippy
cargo fmt --check
```

Frontend:

```bash
cargo check
```

---

# Git Commit Convention

Use:

```
feat: add xxx
fix: fix xxx
refactor: improve xxx
docs: update documentation
test: add tests
chore: maintenance
```

Examples:

```
feat: add firmware download manager
fix: resolve usb connection issue
```

---

# Security Rules

* Do not store secrets in source code
* Validate all external input
* Avoid executing arbitrary commands
* Minimize Tauri permissions
* Keep filesystem access restricted

---

# AI Agent Behavior Rules

When modifying code:

1. Explain the planned changes first
2. Modify only required files
3. Preserve existing architecture
4. Prefer simple solutions
5. Add comments only when necessary
6. Do not rewrite working code without reason
7. Ask before large architectural changes
