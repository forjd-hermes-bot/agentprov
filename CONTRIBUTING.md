# Contributing

Thanks for considering a contribution to AgentProv.

## Development setup

```bash
git clone https://github.com/forjd/agentprov.git
cd agentprov
cargo test
```

## Quality gates

Run these before opening a PR:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Contribution style

- Keep changes small and focused.
- Add tests for behaviour changes.
- Update docs/specs when schema or CLI behaviour changes.
- Prefer portable, dependency-light Rust where practical.
