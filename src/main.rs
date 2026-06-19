use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Parser, Debug)]
#[command(name = "agentprov")]
#[command(about = "MVP identity and provenance primitives for AI agent runs")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Work with agent manifests
    Manifest {
        #[command(subcommand)]
        command: ManifestCommand,
    },
    /// Work with run envelopes
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    /// Work with provenance events
    Event {
        #[command(subcommand)]
        command: EventCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ManifestCommand {
    /// Print an example manifest
    Example,
    /// Hash a manifest file as canonical JSON
    Hash { file: PathBuf },
}

#[derive(Subcommand, Debug)]
enum RunCommand {
    /// Print an example run envelope
    Example,
}

#[derive(Subcommand, Debug)]
enum EventCommand {
    /// Print the canonical BLAKE3 hash for an event file, excluding event_hash
    Hash { file: PathBuf },
    /// Verify an event file's event_hash field
    Verify { file: PathBuf },
}

#[derive(Debug, Serialize, Deserialize)]
struct Owner {
    #[serde(rename = "type")]
    owner_type: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Source {
    repo: String,
    commit: String,
    image_digest: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeManifest {
    #[serde(rename = "type")]
    runtime_type: String,
    environment: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PolicyRef {
    id: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AgentManifest {
    schema: String,
    agent_id: String,
    name: String,
    description: String,
    version: String,
    owner: Owner,
    source: Source,
    runtime: RuntimeManifest,
    capabilities: Vec<String>,
    policy: PolicyRef,
    public_key: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Trigger {
    #[serde(rename = "type")]
    trigger_type: String,
    id: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RunAgentRef {
    agent_id: String,
    version: String,
    manifest_digest: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Actor {
    #[serde(rename = "type")]
    actor_type: String,
    id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    auth_method: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeRun {
    host: String,
    os: String,
    environment: String,
    container_image_digest: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Authority {
    capabilities: Vec<String>,
    policy_id: String,
    policy_version: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RunEnvelope {
    schema: String,
    run_id: String,
    trace_id: String,
    parent_run_id: Option<String>,
    trigger: Trigger,
    agent: RunAgentRef,
    actor_chain: Vec<Actor>,
    runtime: RuntimeRun,
    authority: Authority,
    started_at: DateTime<Utc>,
    ended_at: Option<DateTime<Utc>>,
    status: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Manifest { command } => match command {
            ManifestCommand::Example => print_json(&example_manifest()?),
            ManifestCommand::Hash { file } => {
                let value = read_json_file(file)?;
                println!("{}", canonical_hash(&value)?);
                Ok(())
            }
        },
        Commands::Run { command } => match command {
            RunCommand::Example => print_json(&example_run()?),
        },
        Commands::Event { command } => match command {
            EventCommand::Hash { file } => {
                let mut value = read_json_file(file)?;
                remove_field(&mut value, "event_hash");
                println!("{}", canonical_hash(&value)?);
                Ok(())
            }
            EventCommand::Verify { file } => {
                let mut value = read_json_file(file)?;
                let expected = value
                    .get("event_hash")
                    .and_then(Value::as_str)
                    .context("event_hash must be present and a string")?
                    .to_owned();
                remove_field(&mut value, "event_hash");
                let actual = canonical_hash(&value)?;
                if expected == actual {
                    println!("ok: event hash verifies");
                    Ok(())
                } else {
                    bail!("event hash mismatch: expected {expected}, actual {actual}")
                }
            }
        },
    }
}

fn example_manifest() -> Result<Value> {
    let manifest = AgentManifest {
        schema: "agentprov.dev/manifest/v1".to_owned(),
        agent_id: format!("agent_{}", Uuid::new_v4().simple()),
        name: "research-agent".to_owned(),
        description: "Researches a topic and drafts a response".to_owned(),
        version: "0.1.0".to_owned(),
        owner: Owner {
            owner_type: "github_user".to_owned(),
            id: "danjdewhurst".to_owned(),
        },
        source: Source {
            repo: "https://github.com/example/research-agent".to_owned(),
            commit: "abc123".to_owned(),
            image_digest: None,
        },
        runtime: RuntimeManifest {
            runtime_type: "cli".to_owned(),
            environment: "local".to_owned(),
        },
        capabilities: vec![
            "web.search".to_owned(),
            "http.get".to_owned(),
            "discord.message.create".to_owned(),
        ],
        policy: PolicyRef {
            id: "policy_research_agent".to_owned(),
            version: "v1".to_owned(),
        },
        public_key: None,
    };
    serde_json::to_value(manifest).context("serialize example manifest")
}

fn example_run() -> Result<Value> {
    let run = RunEnvelope {
        schema: "agentprov.dev/run-envelope/v1".to_owned(),
        run_id: format!("run_{}", Uuid::new_v4().simple()),
        trace_id: format!("trace_{}", Uuid::new_v4().simple()),
        parent_run_id: None,
        trigger: Trigger {
            trigger_type: "manual".to_owned(),
            id: "discord_message_123".to_owned(),
        },
        agent: RunAgentRef {
            agent_id: "agent_01hxexample".to_owned(),
            version: "0.1.0".to_owned(),
            manifest_digest: "blake3:example".to_owned(),
        },
        actor_chain: vec![
            Actor {
                actor_type: "user".to_owned(),
                id: "danjdewhurst".to_owned(),
                auth_method: Some("discord".to_owned()),
            },
            Actor {
                actor_type: "service".to_owned(),
                id: "hermes".to_owned(),
                auth_method: None,
            },
            Actor {
                actor_type: "agent".to_owned(),
                id: "agent_01hxexample".to_owned(),
                auth_method: None,
            },
        ],
        runtime: RuntimeRun {
            host: "example-host".to_owned(),
            os: "linux".to_owned(),
            environment: "local".to_owned(),
            container_image_digest: None,
        },
        authority: Authority {
            capabilities: vec!["web.search".to_owned(), "discord.message.create".to_owned()],
            policy_id: "policy_research_agent".to_owned(),
            policy_version: "v1".to_owned(),
        },
        started_at: Utc::now(),
        ended_at: None,
        status: "running".to_owned(),
    };
    serde_json::to_value(run).context("serialize example run")
}

fn read_json_file(path: PathBuf) -> Result<Value> {
    let content =
        fs::read_to_string(&path).with_context(|| format!("read JSON file {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse JSON file {}", path.display()))
}

fn print_json(value: &Value) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn canonical_hash(value: &Value) -> Result<String> {
    let canonical = canonicalise(value);
    let bytes = serde_json::to_vec(&canonical).context("serialise canonical JSON")?;
    let hash = blake3::hash(&bytes);
    Ok(format!("blake3:{}", hash.to_hex()))
}

fn remove_field(value: &mut Value, field: &str) {
    if let Value::Object(map) = value {
        map.remove(field);
    }
}

fn canonicalise(value: &Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut sorted = Map::new();
            let mut keys: Vec<_> = map.keys().collect();
            keys.sort();
            for key in keys {
                if let Some(v) = map.get(key) {
                    sorted.insert(key.clone(), canonicalise(v));
                }
            }
            Value::Object(sorted)
        }
        Value::Array(values) => Value::Array(values.iter().map(canonicalise).collect()),
        _ => value.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn canonical_hash_is_independent_of_object_key_order() {
        let a = json!({"b": 2, "a": 1, "nested": {"z": true, "c": null}});
        let b = json!({"nested": {"c": null, "z": true}, "a": 1, "b": 2});
        assert_eq!(canonical_hash(&a).unwrap(), canonical_hash(&b).unwrap());
    }

    #[test]
    fn event_hash_excludes_event_hash_field() {
        let mut event = json!({"event_hash": "ignored", "sequence": 1});
        remove_field(&mut event, "event_hash");
        assert_eq!(event, json!({"sequence": 1}));
    }
}
