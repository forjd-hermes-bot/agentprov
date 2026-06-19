# Policy v1

## Purpose

A Policy defines which actions an agent may perform, which are denied, and which require approval.

## Required fields

- `schema`: must be `agentprov.dev/policy/v1`
- `policy_id`
- `version`
- `agent_id`

## Rule lists

- `allow`
- `deny`
- `require_approval`

Each rule has:

- `action`
- `resource`

Future versions may add `expires_at`, `conditions`, and richer resource matching.

## Matching rules

The MVP supports:

- exact matches
- `*` wildcard
- prefix wildcard suffix, for example `discord://guild/123/*`

## Decision order

1. If the policy `agent_id` does not match, deny.
2. Deny rules win.
3. Require-approval rules are distinct from allow.
4. Allow rules allow.
5. Otherwise deny.
