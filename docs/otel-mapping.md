# OpenTelemetry and OpenInference Mapping

AgentProv should be compatible with existing GenAI observability tools.

## Strategy

- Use OpenTelemetry trace/span IDs where possible.
- Map agent, tool and model events to OpenTelemetry GenAI semantic conventions.
- Map AI-specific spans to OpenInference span kinds.
- Add AgentProv-specific attributes only for identity, authority and verification fields that existing conventions do not cover.

## OpenInference mapping

| AgentProv event | OpenInference span kind |
| --- | --- |
| `agent.invoke` | `AGENT` |
| `agent.plan` | `AGENT` or `CHAIN` |
| `llm.call` | `LLM` |
| `tool.execute` | `TOOL` |
| `prompt.render` | `PROMPT` |
| `retrieval.search` | `RETRIEVER` |
| `guardrail.check` | `GUARDRAIL` |
| `eval.run` | `EVALUATOR` |
| `permission.check` | custom event/span |
| `human.approval.*` | custom event/span |
| `secret.access` | custom event/span |
| `artifact.*` | custom event/span |

## OpenTelemetry GenAI mapping

| AgentProv field | OTel GenAI field |
| --- | --- |
| `agent.id` | `gen_ai.agent.id` |
| `agent.name` | `gen_ai.agent.name` |
| `agent.version` | `gen_ai.agent.version` |
| `agent.description` | `gen_ai.agent.description` |
| `tool.name` | `gen_ai.tool.name` |
| `tool.type` | `gen_ai.tool.type` |
| `tool.call_id` | `gen_ai.tool.call.id` |
| `operation = agent.invoke` | `gen_ai.operation.name = invoke_agent` |
| `operation = tool.execute` | `gen_ai.operation.name = execute_tool` |
| `operation = agent.plan` | `gen_ai.operation.name = plan` |
| `memory.read` | `gen_ai.operation.name = search_memory` |
| `memory.write` | `gen_ai.operation.name = update_memory` or `upsert_memory` |

## AgentProv extension attributes

Suggested namespace: `agentprov.*`

Identity:

- `agentprov.agent.manifest_digest`
- `agentprov.agent.key_id`
- `agentprov.agent.owner`
- `agentprov.agent.issuer`
- `agentprov.runtime.id`
- `agentprov.runtime.host`
- `agentprov.runtime.container_image_digest`
- `agentprov.runtime.environment`

Trigger:

- `agentprov.trigger.type`
- `agentprov.trigger.id`
- `agentprov.schedule.id`
- `agentprov.webhook.id`
- `agentprov.ci.job_id`

Actor chain:

- `agentprov.actor_chain`
- `agentprov.delegated_user.id`
- `agentprov.parent_run.id`

Permissions:

- `agentprov.permission.subject`
- `agentprov.permission.action`
- `agentprov.permission.resource`
- `agentprov.permission.scope`
- `agentprov.permission.policy_id`
- `agentprov.permission.policy_version`
- `agentprov.permission.decision`
- `agentprov.permission.reason`
- `agentprov.permission.approval_id`

Verification:

- `agentprov.event.sequence`
- `agentprov.event.hash`
- `agentprov.event.previous_hash`
- `agentprov.event.signature`
- `agentprov.event.key_id`
- `agentprov.schema.version`

## Payload handling

Default behaviour should be:

- Store payload digests.
- Store redacted previews only where useful.
- Make full prompt/input/output capture opt-in.
- Allow object storage references for larger payloads.
- Record redaction policy/version so audits know what was hidden and why.
