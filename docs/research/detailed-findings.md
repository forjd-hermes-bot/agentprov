# Detailed Findings From Existing Open Source Projects

## Langfuse

Repository: https://github.com/langfuse/langfuse

Langfuse is an open source LLM engineering and observability platform.

Useful product primitives:

- trace
- session
- observation
- prompt management
- evaluations
- datasets
- playground

Useful data model ideas:

- traces include project, session, user, environment, release, version, tags, metadata, input and output
- observations include parent observation IDs, typed observation kinds, metadata, latency, cost, usage and prompt linkage
- observation types include span, event, generation, agent, tool, chain, retriever, evaluator, embedding and guardrail

Useful architecture ideas:

- ClickHouse for high-volume trace/observation analytics
- Postgres for application state
- Redis for API key and prompt caching
- project-scoped API keys
- MCP server for programmatic access to traces and prompts

Gap:

Langfuse is very good at observing agent/tool/model behaviour, but agent identity is not a first-class signed or permission-bearing principal.

## Arize Phoenix

Repository: https://github.com/Arize-ai/phoenix

Phoenix is an open source AI observability platform with tracing, evaluation, prompt management, experiments and replay/debugging workflows.

Useful ideas:

- OpenTelemetry-based LLM traces
- integrations for OpenAI Agents, Claude Agent SDK, LangGraph, Google ADK, LiteLLM, MCP, AutoGen and others
- trace trees for agent, tool, retriever, embedding and LLM operations
- prompt management and replay from traces

Gap:

Phoenix is a strong trace backend and UI, but it does not define a full identity/authority/provenance layer for agents.

## OpenInference

Repository: https://github.com/Arize-ai/openinference

OpenInference is a set of conventions and plugins complementary to OpenTelemetry for tracing AI applications.

Useful span kinds:

- LLM
- EMBEDDING
- CHAIN
- RETRIEVER
- RERANKER
- TOOL
- AGENT
- GUARDRAIL
- EVALUATOR
- PROMPT

Useful attributes:

- `input.value`
- `output.value`
- `metadata`
- `session.id`
- `user.id`
- `llm.input_messages`
- `llm.output_messages`
- `llm.invocation_parameters`
- `llm.provider`
- `llm.model_name`
- `llm.tools`
- `llm.cost.*`
- `llm.token_count.*`
- `agent.name`
- `tool.name`
- `tool.id`
- `tool.parameters`
- `tool.json_schema`

Gap:

OpenInference gives useful AI trace semantics, but not a cryptographic identity, permission or actor-chain model.

## OpenTelemetry GenAI semantic conventions

Repository: https://github.com/open-telemetry/semantic-conventions-genai

Useful standard concepts:

- `gen_ai.operation.name`
- `create_agent`
- `invoke_agent`
- `execute_tool`
- `plan`
- `create_memory`
- `search_memory`
- `update_memory`
- `delete_memory`

Useful attributes:

- `gen_ai.agent.id`
- `gen_ai.agent.name`
- `gen_ai.agent.version`
- `gen_ai.agent.description`
- `gen_ai.tool.name`
- `gen_ai.tool.type`
- `gen_ai.tool.call.id`
- `gen_ai.tool.description`
- `gen_ai.memory.store.id`
- `gen_ai.memory.record.id`

Important design note:

The OTel GenAI conventions treat prompts, model inputs, outputs, tool arguments and tool results as sensitive. AgentProv should follow that by storing digests/redacted previews by default and making full payload capture opt-in.

Gap:

OTel GenAI provides telemetry semantics, not a full provenance/security model.

## AgentOps

Repository: https://github.com/AgentOps-AI/agentops

AgentOps is an observability and devtool platform for AI agents.

Useful span hierarchy:

- SESSION
- AGENT
- WORKFLOW
- OPERATION
- TASK
- LLM
- TOOL

Useful integration ideas:

- Python SDK auto-instrumentation
- decorators such as `@agent`, `@operation` and `@trace`
- OpenTelemetry exporter support
- execution graph/session replay UI
- cost management

Useful semantic ideas:

- agent ID
- agent name
- agent role
- tool name
- tool parameters
- tool result
- trace/span/parent IDs

Gap:

AgentOps is the closest agent-native reference, but its agent identity is still primarily telemetry metadata rather than a registered, signed, revocable principal with capabilities.

## Helicone

Repository: https://github.com/Helicone/helicone

Helicone is an open source LLM observability platform and AI gateway.

Useful adoption ideas:

- OpenAI-compatible gateway
- one-line SDK integration via base URL replacement
- metadata via headers
- custom properties
- predefined request IDs
- sessions and session paths
- provider routing and fallbacks

Useful headers and metadata patterns:

- `Helicone-Request-Id`
- `Helicone-User-Id`
- `Helicone-Session-Id`
- `Helicone-Session-Path`
- `Helicone-Session-Name`
- `Helicone-Prompt-Id`
- `Helicone-Model-Override`
- `Helicone-Property-[Name]`

Gap:

Headers and custom properties are flexible, but they do not by themselves provide typed, verified, signed provenance or fine-grained per-agent permissions.

## MLflow

Repository: https://github.com/mlflow/mlflow

MLflow now includes LLM and agent tracing, evaluation, prompt registry, AI gateway and monitoring capabilities.

Useful ideas:

- OpenTelemetry-compatible tracing
- `TraceInfo` for metadata
- `TraceData` for spans
- span inputs, outputs, attributes and events
- lightweight metadata in database tables
- large/binary attachments in artifact storage
- prompt registry and evaluation workflows

Gap:

MLflow provides a broad AI engineering platform, but agent identity, delegation, capabilities and signed run records are not its core abstraction.

## W&B Weave

Repository: https://github.com/wandb/weave

Weave is a toolkit for developing generative AI applications with tracing, evaluations and workflow organisation.

Useful ideas:

- `@weave.op` traces arbitrary functions
- trace tree of inputs and outputs
- object/version/tag/alias model
- call start/end/update APIs
- OTel ingest endpoints
- agent spans query/stats APIs
- feedback APIs

Gap:

Weave is strong for developer tracing and object/version tracking, but not a permission-aware provenance layer.
