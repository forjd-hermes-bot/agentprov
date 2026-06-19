# AgentProv OpenTelemetry Attributes v1

## Purpose

AgentProv should interoperate with OpenTelemetry GenAI and OpenInference rather than defining a closed observability format.

## Attribute namespace

AgentProv-specific fields use the `agentprov.*` namespace.

## Identity attributes

- `agentprov.agent.manifest_digest`
- `agentprov.agent.key_id`
- `agentprov.agent.owner`
- `agentprov.agent.issuer`
- `agentprov.runtime.id`
- `agentprov.runtime.host`
- `agentprov.runtime.container_image_digest`
- `agentprov.runtime.environment`

## Trigger attributes

- `agentprov.trigger.type`
- `agentprov.trigger.id`
- `agentprov.schedule.id`
- `agentprov.webhook.id`
- `agentprov.ci.job_id`

## Actor chain attributes

- `agentprov.actor_chain`
- `agentprov.delegated_user.id`
- `agentprov.parent_run.id`

## Permission attributes

- `agentprov.permission.subject`
- `agentprov.permission.action`
- `agentprov.permission.resource`
- `agentprov.permission.scope`
- `agentprov.permission.policy_id`
- `agentprov.permission.policy_version`
- `agentprov.permission.decision`
- `agentprov.permission.reason`
- `agentprov.permission.approval_id`

## Verification attributes

- `agentprov.event.sequence`
- `agentprov.event.hash`
- `agentprov.event.previous_hash`
- `agentprov.event.signature`
- `agentprov.event.key_id`
- `agentprov.schema.version`

## Existing convention mappings

- `agent_id` maps to `gen_ai.agent.id`
- `agent.name` maps to `gen_ai.agent.name`
- `agent.version` maps to `gen_ai.agent.version`
- `tool.name` maps to `gen_ai.tool.name`
- `tool.call_id` maps to `gen_ai.tool.call.id`
- `tool.execute` maps to `gen_ai.operation.name = execute_tool`
- `agent.invoke` maps to `gen_ai.operation.name = invoke_agent`

## OpenInference span kinds

- `llm.call` -> `LLM`
- `tool.execute` -> `TOOL`
- `prompt.render` -> `PROMPT`
- `run.start` / `agent.invoke` -> `AGENT`
- `retrieval.search` -> `RETRIEVER`
- `guardrail.check` -> `GUARDRAIL`
- `eval.run` -> `EVALUATOR`
