# AgentProv

AgentProv is an MVP Rust project for open, verifiable identity and provenance records for AI agent runs.

The core idea: existing LLM observability tools are good at showing what happened, but they rarely answer the harder audit questions:

- Who or what ran the agent?
- Where did it run?
- Which model and provider were used?
- What prompt/template/version was used?
- Was it manual, scheduled, webhook-triggered, CI-triggered, or delegated by another agent?
- What permissions and tools were available?
- Which permission checks allowed or denied actions?
- Can the resulting record be verified later?

AgentProv is intended to sit beside tools like Langfuse, Phoenix/OpenInference, AgentOps, Helicone, MLflow and Weave rather than replace them.

## MVP contents

This repository currently contains:

- Research notes in `docs/research/`
- Product scope in `docs/mvp-scope.md`
- Schema sketches in `docs/schemas/`
- OpenTelemetry/OpenInference mapping notes in `docs/otel-mapping.md`
- A small Rust CLI that can:
  - create an example agent manifest
  - create an example run envelope
  - hash a provenance event using canonical JSON
  - verify an event hash

## Quick start

```bash
cargo run -- manifest example
cargo run -- run example
cargo run -- event hash examples/event.json
cargo test
```

## Local quality gates

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Positioning

Langfuse, Phoenix, AgentOps, Helicone, MLflow and Weave show what happened in an AI system.

AgentProv aims to answer whether the actor, authority, runtime and event chain are trustworthy.

## License

MIT OR Apache-2.0
