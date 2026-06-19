# Threat Model

## Threats AgentProv should address

- An organisation cannot tell which agent performed an action.
- A log says `agent_name = x`, but that value was just user-controlled metadata.
- A scheduled run and a manual run are indistinguishable in the audit trail.
- A subagent performs an action and the parent/user delegation chain is lost.
- A model requests a tool call and the system cannot prove whether policy allowed it.
- A dangerous action occurs without recording whether human approval was required or granted.
- A run used a different model, provider, prompt or toolset than expected.
- A prompt/output/tool result was modified in logs after the fact.
- Sensitive prompt/tool payloads are stored without clear redaction policy.

## Not solved in the MVP

- A malicious root user on the runtime host.
- Fully confidential computing or enclave attestation.
- Perfect non-repudiation across all deployment types.
- Complete DLP and secret detection.
- Preventing a model from making a bad decision.
- Replacing application-level authorisation.
- Replacing full SIEM/security monitoring.

## MVP security posture

The MVP should provide tamper-evident records, not tamper-proof infrastructure.

It should make accidental or casual log mutation detectable by:

- canonical event serialisation
- event hashes
- previous-event hash chaining
- optional signatures
- explicit key IDs
- exportable verification

## Privacy posture

Default to less data:

- prompt digest instead of full prompt
- tool input/output digest instead of full payload
- redacted previews where useful
- explicit capture mode for full payloads
- redaction policy/version recorded in metadata
