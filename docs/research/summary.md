# Research Summary

## Repositories inspected

- Langfuse: https://github.com/langfuse/langfuse
- Arize Phoenix: https://github.com/Arize-ai/phoenix
- OpenInference: https://github.com/Arize-ai/openinference
- OpenTelemetry GenAI semantic conventions: https://github.com/open-telemetry/semantic-conventions-genai
- AgentOps: https://github.com/AgentOps-AI/agentops
- Helicone: https://github.com/Helicone/helicone
- MLflow: https://github.com/mlflow/mlflow
- W&B Weave: https://github.com/wandb/weave

## Main finding

The open source ecosystem has strong LLM and agent observability foundations, especially around traces, spans, sessions, tool calls, model calls, prompt versions, token/cost metrics and evaluation.

It does not yet have a dominant open source layer for verifiable agent identity, actor chains, capability grants, permission decisions, runtime provenance and tamper-evident run records.

## What to reuse

### Trace/session envelope

Langfuse, Phoenix, AgentOps, MLflow and Weave all model work as traces/sessions/runs with nested spans or observations.

AgentProv should reuse that shape:

- run = high-level provenance envelope
- event/span = individual action
- parent IDs = nested causality
- session/conversation IDs = grouping across runs

### Typed span model

Reusable span kinds from OpenInference and AgentOps:

- AGENT
- LLM
- TOOL
- PROMPT
- CHAIN / WORKFLOW
- RETRIEVER
- EVALUATOR
- GUARDRAIL
- TASK / OPERATION

AgentProv should add identity and authority-specific events:

- permission.check
- capability.grant
- human.approval.request
- human.approval.grant
- human.approval.deny
- secret.access
- data.access
- artifact.create
- artifact.update
- artifact.delete
- delegation.create

### Header/gateway adoption

Helicone shows that adoption improves when metadata can be sent through an OpenAI-compatible gateway with simple headers.

Future AgentProv gateways could accept headers such as:

- AgentProv-Agent-Id
- AgentProv-Run-Id
- AgentProv-Parent-Run-Id
- AgentProv-Trigger-Type
- AgentProv-Capabilities
- AgentProv-Policy-Id

### Decorators and context managers

AgentOps, MLflow and Weave show the value of decorators/context managers.

Future SDKs should expose:

- `@agent`
- `@tool`
- `@workflow`
- `@requires_permission`
- `with agent_run(...)`
- `with permission_check(...)`

### OpenTelemetry compatibility

Phoenix, OpenInference, AgentOps, MLflow and Weave all lean towards OpenTelemetry compatibility.

AgentProv should not create a closed trace format. It should define extension attributes and event schemas that can be exported as OTLP spans.

## Key gap

Existing systems mostly treat agent identity as metadata.

AgentProv should treat agent identity as a principal that can:

- have a manifest
- own a public key
- be versioned
- be revoked
- hold capabilities
- request permissions
- delegate work
- sign run events

## Comparison table

| Project | Strong at | Useful idea | Gap for AgentProv |
| --- | --- | --- | --- |
| Langfuse | LLM tracing, prompt management, sessions | Trace/observation model, prompt versions | Agent identity is not a verified principal |
| Phoenix | AI tracing and evals | OpenInference support, trace UI | Provenance/security layer is thin |
| OpenInference | AI span conventions | AGENT/TOOL/LLM/PROMPT semantic model | No permissions or signed identity |
| OTel GenAI | Standardising GenAI telemetry | `gen_ai.agent.*`, `execute_tool`, memory ops | No actor chain or signed provenance |
| AgentOps | Agent-native tracing | SESSION/AGENT/TOOL/LLM hierarchy, decorators | Agent ID is telemetry metadata |
| Helicone | Gateway adoption and request logging | Header metadata, sessions, request IDs | Metadata is not strongly typed or signed |
| MLflow | AI engineering lifecycle | TraceInfo/TraceData split, artifact storage | Less focused on runtime agent authority |
| Weave | Function/call tracing | `@weave.op`, object versions, call APIs | Provenance is not permission-aware |
