# AgentProv MVP Scope

## One-line thesis

AgentProv is an open source identity, provenance and permission layer for AI agent runs, built around OpenTelemetry-compatible traces and signed provenance events.

## Problem

AI agent observability tools can usually answer:

- what model was called
- what prompt was sent
- what tool was invoked
- how long it took
- how much it cost
- what the model returned

They are weaker at answering:

- who or what initiated the run
- whether it was manual, scheduled, webhook-triggered, CI-triggered, or delegated
- where the run executed
- which exact agent implementation/version ran
- what permissions and secrets were available
- whether a tool call was authorised by policy
- whether a human approved a risky action
- whether the audit record was modified afterwards

## Goal

Create a small Rust-first MVP that defines the core records and verification model for trustworthy agent runs.

The MVP should not become another LLM observability dashboard. It should be the identity/provenance layer that can export to, or sit beside, observability systems.

## Core concepts

### Agent Manifest

A signed description of an agent as a principal.

Captures:

- stable `agent_id`
- name and description
- owner/org/project
- version
- source repository and commit
- container image digest or build ID where available
- runtime type
- public key/key ID
- manifest digest
- declared capabilities
- allowed model providers/models
- policy reference

### Run Envelope

One execution of an agent.

Captures:

- `run_id`
- `trace_id`
- optional `parent_run_id`
- trigger type: manual, scheduled, webhook, api, ci, delegated
- trigger ID, such as schedule ID, webhook ID or CI job ID
- actor chain
- agent ID/version
- runtime host/container/environment
- available tools and capabilities
- model/provider configuration
- prompt/template version or digest
- policy version
- start/end/status

### Actor Chain

A formal chain of responsibility.

Example:

```text
human user -> service -> agent -> subagent -> tool -> external system
```

This avoids flattening everything into a single `user.id` or `agent.name`.

### Permission Decision

A first-class record of authority.

Captures:

- subject
- action
- resource
- scope
- policy ID/version
- decision: allow, deny, abstain
- reason
- expiry
- approval ID where relevant

### Provenance Event

An append-only event in a run.

Captures:

- event ID
- run ID
- sequence number
- event type
- timestamp
- subject/actor
- resource/action
- payload digest or redacted payload reference
- previous event hash
- event hash
- signature/key ID

## MVP in scope

- Rust CLI crate
- JSON schema sketches for manifest, run envelope, event and policy
- canonical JSON hashing for events
- example manifest/run/event fixtures
- verification command for event hashes
- research docs from existing OSS tools
- OpenTelemetry/OpenInference mapping notes
- static YAML/JSON-like policy design notes
- high-level implementation roadmap

## MVP out of scope

- hosted SaaS
- full web UI
- complex RBAC/SSO
- evals/datasets/prompt playground
- full OpenTelemetry collector implementation
- production-grade cryptographic key management
- transparency log
- confidential computing
- complete DLP/redaction system

## Product principles

1. OpenTelemetry-compatible, not a closed format.
2. Payload-light by default: digests and redacted previews before full prompt/output capture.
3. Agent identity is a principal, not a string in trace metadata.
4. Every important action should have a permission decision.
5. Event chains should be tamper-evident and exportable.
6. Existing observability products are integration targets, not competitors.

## First demo target

A minimal demo should:

1. Load or generate an agent manifest.
2. Start a run envelope with `trigger_type = manual`.
3. Record an LLM call event.
4. Record a permission check for a tool call.
5. Record the tool execution event.
6. Hash the events as a chain.
7. Verify the chain from the CLI.

A good demo sentence:

> Dan triggered `research-agent` v0.1.0 from Discord. It ran on host X using OpenAI model Y. It had web search and Discord reply capabilities. It called web search, checked permission to post a message, posted to Discord, and the event chain verifies.
