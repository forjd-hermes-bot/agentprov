# Implementation Roadmap

## Phase 0: Repository and research

- Rust CLI crate
- docs/research findings
- MVP scope
- schema sketches
- OTel/OpenInference mapping
- threat model

## Phase 1: Core data structures

- `AgentManifest`
- `RunEnvelope`
- `Actor`
- `PermissionDecision`
- `ProvenanceEvent`
- canonical JSON hashing
- event verification

## Phase 2: Event chains

- append events to a run log
- validate sequence numbers
- validate previous hash links
- verify whole run chains
- add example fixtures

## Phase 3: Signing

- generate local Ed25519 keypair
- sign manifest digest
- sign event hashes
- verify signatures
- document key handling limitations

## Phase 4: Policy MVP

- static policy file
- allow/deny/require approval rules
- CLI policy check command
- permission decision event generation

## Phase 5: SDK shape

- Rust library API
- Python SDK prototype
- TypeScript SDK prototype
- decorators/context managers in dynamic SDKs

## Phase 6: OpenTelemetry export

- map provenance events to OTLP spans
- map AgentProv fields to OTel GenAI and OpenInference attributes
- export examples for Phoenix/Jaeger/Tempo

## Phase 7: Collector

- local HTTP ingest server
- SQLite/Postgres persistence
- query API for runs/events
- verification endpoint

## Phase 8: UI

- run list
- run detail
- actor chain view
- permission timeline
- trace/event tree
- verification status

## Phase 9: Integrations

- OpenAI/Anthropic/LiteLLM wrappers
- Langfuse/Phoenix export examples
- AgentOps/Helicone interop notes
- GitHub/Discord tool examples
- scheduled run example
