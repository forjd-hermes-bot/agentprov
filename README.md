# AgentProv

AgentProv is a Rust MVP for signed provenance records for AI agent runs.

Existing LLM observability tools are good at showing what happened: prompts, model calls, tool calls, latency, cost and traces. AgentProv focuses on the next audit question:

> Who or what ran this agent, with what authority, where did it run, and does the record still verify?

## What AgentProv is for

AgentProv is intended to sit beside tools like Langfuse, Phoenix/OpenInference, AgentOps, Helicone, MLflow and Weave rather than replace them.

Those tools show what happened. AgentProv aims to prove:

- which agent identity ran
- which trigger started the run
- which actor chain led to the action
- which capabilities and policies were available
- which permission checks allowed or denied actions
- whether the run log was modified afterwards

## 30-second demo

```bash
cargo run -- demo manual-tool-run --out demo-output/
cargo run -- run verify demo-output/run.jsonl
```

Expected shape:

```text
Run verifies
Events: 4
Event chain: valid
Signatures: not present
```

## Current CLI

```bash
# Examples
cargo run -- manifest example
cargo run -- run example

# Event hashing and verification
cargo run -- event hash examples/event.json
cargo run -- event verify examples/event.json

# Append-only run logs
cargo run -- run init --agent examples/manifest.json --trigger manual --out runs/run_123.jsonl
cargo run -- event append --run runs/run_123.jsonl --type permission.check --action discord.message.create --resource discord://guild/123/channel/456
cargo run -- run verify runs/run_123.jsonl

# Local MVP signing
cargo run -- key generate --out agentprov.key
cargo run -- event sign examples/event.json --key agentprov.key --out event.signed.json
cargo run -- event verify-signature event.signed.json

# Static policy checks
cargo run -- policy check --policy examples/policy.json --agent agent_01hxexample --action discord.message.create --resource discord://guild/148756/channel/456

# Export experiments
cargo run -- export otel demo-output/run.jsonl --out run.otlp.json
cargo run -- export openinference demo-output/run.jsonl --out run.openinference.json
```

## Documentation

- `docs/research/summary.md` — short research summary
- `docs/research/detailed-findings.md` — findings from existing OSS tools
- `docs/mvp-scope.md` — MVP product scope
- `docs/next-steps.md` — implementation plan
- `docs/roadmap.md` — longer roadmap
- `docs/threat-model.md` — threat model
- `docs/otel-mapping.md` — OpenTelemetry/OpenInference mapping notes
- `docs/spec/` — versioned spec docs
- `schemas/` — machine-readable JSON Schemas

## Local quality gates

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo build --release
```

## Status

This is an early MVP. Key handling is for local experimentation only, not production key management.

## License

MIT
