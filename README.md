# ig-agent

Minimal Rust CLI skeleton for a future Instagram content publisher.

V1 is intentionally locked down. It does not call Meta, OpenAI or any external API.

## Commands

```powershell
$env:POSTS_CONFIG="assets/posts.toml"
$env:POST_LOG="logs/posts.jsonl"
$env:DRY_RUN="true"

cargo run -- validate-config
cargo run -- dry-run
```

## Verification

```powershell
cargo generate-lockfile
cargo check --locked
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo tree
```

Commit `Cargo.lock` after generating it.
