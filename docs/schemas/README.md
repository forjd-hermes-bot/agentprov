# JSON Schema Sketches

These are early, human-readable schema sketches. They are not yet formal JSON Schema documents.

## Agent Manifest

```json
{
  "schema": "agentprov.dev/manifest/v1",
  "agent_id": "agent_01hxexample",
  "name": "research-agent",
  "description": "Researches a topic and drafts a response",
  "version": "0.1.0",
  "owner": {
    "type": "github_user",
    "id": "danjdewhurst"
  },
  "source": {
    "repo": "https://github.com/example/research-agent",
    "commit": "abc123",
    "image_digest": "sha256:optional"
  },
  "runtime": {
    "type": "cli",
    "environment": "local"
  },
  "models": {
    "allowed_providers": ["openai", "anthropic"],
    "allowed_models": ["gpt-5.5", "claude-sonnet-4.5"]
  },
  "capabilities": [
    "web.search",
    "http.get",
    "discord.message.create"
  ],
  "policy": {
    "id": "policy_research_agent",
    "version": "v1"
  },
  "public_key": {
    "key_id": "key_01hxexample",
    "algorithm": "ed25519",
    "public_key": "base64-or-hex"
  }
}
```

## Run Envelope

```json
{
  "schema": "agentprov.dev/run-envelope/v1",
  "run_id": "run_01hxexample",
  "trace_id": "trace_01hxexample",
  "parent_run_id": null,
  "trigger": {
    "type": "manual",
    "id": "discord_message_123"
  },
  "agent": {
    "agent_id": "agent_01hxexample",
    "version": "0.1.0",
    "manifest_digest": "blake3:..."
  },
  "actor_chain": [
    { "type": "user", "id": "danjdewhurst", "auth_method": "discord" },
    { "type": "service", "id": "hermes" },
    { "type": "agent", "id": "agent_01hxexample" }
  ],
  "runtime": {
    "host": "hostname",
    "os": "linux",
    "environment": "local",
    "container_image_digest": null
  },
  "authority": {
    "capabilities": ["web.search", "discord.message.create"],
    "policy_id": "policy_research_agent",
    "policy_version": "v1"
  },
  "started_at": "2026-06-19T10:00:00Z",
  "ended_at": null,
  "status": "running"
}
```

## Provenance Event

```json
{
  "schema": "agentprov.dev/event/v1",
  "event_id": "evt_01hxexample",
  "run_id": "run_01hxexample",
  "sequence": 1,
  "timestamp": "2026-06-19T10:00:01Z",
  "event_type": "permission.check",
  "subject": {
    "type": "agent",
    "id": "agent_01hxexample"
  },
  "action": "discord.message.create",
  "resource": "discord://guild/148756/channel/148756",
  "payload_digest": "blake3:...",
  "previous_event_hash": null,
  "event_hash": "blake3:...",
  "signature": null,
  "key_id": "key_01hxexample"
}
```

## Policy

```json
{
  "schema": "agentprov.dev/policy/v1",
  "policy_id": "policy_research_agent",
  "version": "v1",
  "agent_id": "agent_01hxexample",
  "allow": [
    {
      "action": "http.get",
      "resource": "https://*.gov.uk/*"
    },
    {
      "action": "discord.message.create",
      "resource": "discord://guild/148756/*"
    }
  ],
  "require_approval": [
    {
      "action": "github.pr.merge",
      "resource": "*"
    }
  ],
  "deny": [
    {
      "action": "secret.read",
      "resource": "prod/*"
    }
  ]
}
```
