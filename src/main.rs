use anyhow::{Context, Result, bail};
use chrono::{DateTime, Utc};
use clap::{Parser, Subcommand, ValueEnum};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand_core::OsRng;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
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
    /// Work with run envelopes and append-only run logs
    Run {
        #[command(subcommand)]
        command: RunCommand,
    },
    /// Work with provenance events
    Event {
        #[command(subcommand)]
        command: EventCommand,
    },
    /// Work with local MVP signing keys
    Key {
        #[command(subcommand)]
        command: KeyCommand,
    },
    /// Work with static policy files
    Policy {
        #[command(subcommand)]
        command: PolicyCommand,
    },
    /// Generate deterministic demos
    Demo {
        #[command(subcommand)]
        command: DemoCommand,
    },
    /// Export run logs to interoperability formats
    Export {
        #[command(subcommand)]
        command: ExportCommand,
    },
}

#[derive(Subcommand, Debug)]
enum ManifestCommand {
    /// Print an example manifest
    Example,
    /// Hash a manifest file as canonical JSON
    Hash { file: PathBuf },
    /// Sign a manifest file with an MVP local key
    Sign {
        file: PathBuf,
        #[arg(long)]
        key: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum RunCommand {
    /// Print an example run envelope
    Example,
    /// Initialise an append-only run log with a run.start event
    Init {
        #[arg(long)]
        agent: PathBuf,
        #[arg(long, value_enum)]
        trigger: TriggerType,
        #[arg(long)]
        out: PathBuf,
    },
    /// Verify an append-only run log
    Verify {
        file: PathBuf,
        #[arg(long)]
        require_signatures: bool,
    },
}

#[derive(Subcommand, Debug)]
enum EventCommand {
    /// Print the canonical BLAKE3 hash for an event file, excluding event_hash/signature
    Hash { file: PathBuf },
    /// Verify an event file's event_hash field
    Verify { file: PathBuf },
    /// Append a provenance event to a run log
    Append {
        #[arg(long)]
        run: PathBuf,
        #[arg(long = "type")]
        event_type: String,
        #[arg(long)]
        action: Option<String>,
        #[arg(long)]
        resource: Option<String>,
        #[arg(long)]
        subject: Option<String>,
    },
    /// Sign an event file with an MVP local key
    Sign {
        file: PathBuf,
        #[arg(long)]
        key: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Verify an event signature using the embedded public key
    VerifySignature { file: PathBuf },
}

#[derive(Subcommand, Debug)]
enum KeyCommand {
    /// Generate an MVP local Ed25519 key file
    Generate {
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum PolicyCommand {
    /// Check a static policy file
    Check {
        #[arg(long)]
        policy: PathBuf,
        #[arg(long)]
        agent: String,
        #[arg(long)]
        action: String,
        #[arg(long)]
        resource: String,
    },
}

#[derive(Subcommand, Debug)]
enum DemoCommand {
    /// Generate a deterministic manual tool-run demo
    ManualToolRun {
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Subcommand, Debug)]
enum ExportCommand {
    /// Export a run log to an OTel-shaped JSON document
    Otel {
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
    /// Export a run log to an OpenInference-shaped JSON document
    Openinference {
        file: PathBuf,
        #[arg(long)]
        out: PathBuf,
    },
}

#[derive(Clone, Debug, ValueEnum)]
enum TriggerType {
    Manual,
    Scheduled,
    Webhook,
    Api,
    Ci,
    Delegated,
}

impl std::fmt::Display for TriggerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value = match self {
            TriggerType::Manual => "manual",
            TriggerType::Scheduled => "scheduled",
            TriggerType::Webhook => "webhook",
            TriggerType::Api => "api",
            TriggerType::Ci => "ci",
            TriggerType::Delegated => "delegated",
        };
        write!(f, "{value}")
    }
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

#[derive(Debug, Serialize, Deserialize)]
struct LocalKeyFile {
    schema: String,
    algorithm: String,
    key_id: String,
    public_key: String,
    secret_key: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Manifest { command } => handle_manifest(command),
        Commands::Run { command } => handle_run(command),
        Commands::Event { command } => handle_event(command),
        Commands::Key { command } => handle_key(command),
        Commands::Policy { command } => handle_policy(command),
        Commands::Demo { command } => handle_demo(command),
        Commands::Export { command } => handle_export(command),
    }
}

fn handle_manifest(command: ManifestCommand) -> Result<()> {
    match command {
        ManifestCommand::Example => print_json(&example_manifest()?),
        ManifestCommand::Hash { file } => {
            let value = read_json_file(&file)?;
            println!("{}", canonical_hash(&value)?);
            Ok(())
        }
        ManifestCommand::Sign { file, key, out } => {
            let mut value = read_json_file(&file)?;
            sign_value(&mut value, &read_key(&key)?)?;
            write_pretty_json(&out, &value)?;
            println!("signed manifest written to {}", out.display());
            Ok(())
        }
    }
}

fn handle_run(command: RunCommand) -> Result<()> {
    match command {
        RunCommand::Example => print_json(&example_run()?),
        RunCommand::Init {
            agent,
            trigger,
            out,
        } => {
            let manifest = read_json_file(&agent)?;
            let run_id = format!("run_{}", Uuid::new_v4().simple());
            let event = build_event(
                run_id,
                1,
                "run.start".to_owned(),
                Some(format!("trigger.{trigger}")),
                Some(agent.display().to_string()),
                None,
                None,
                Some(json!({
                    "agent_manifest_digest": canonical_hash(&manifest)?,
                    "trigger_type": trigger.to_string(),
                })),
            )?;
            write_jsonl(&out, &[event])?;
            println!("run log initialised at {}", out.display());
            Ok(())
        }
        RunCommand::Verify {
            file,
            require_signatures,
        } => {
            let report = verify_run_log(&file, require_signatures)?;
            println!("Run verifies");
            println!("Events: {}", report.events);
            println!("Event chain: valid");
            println!(
                "Signatures: {}",
                if report.signatures_present {
                    "valid"
                } else {
                    "not present"
                }
            );
            Ok(())
        }
    }
}

fn handle_event(command: EventCommand) -> Result<()> {
    match command {
        EventCommand::Hash { file } => {
            let value = read_json_file(&file)?;
            println!("{}", event_hash(&value)?);
            Ok(())
        }
        EventCommand::Verify { file } => {
            verify_event_hash(&read_json_file(&file)?)?;
            println!("ok: event hash verifies");
            Ok(())
        }
        EventCommand::Append {
            run,
            event_type,
            action,
            resource,
            subject,
        } => {
            let events = read_jsonl(&run)?;
            let last = events
                .last()
                .with_context(|| format!("run log {} has no events", run.display()))?;
            let run_id = last
                .get("run_id")
                .and_then(Value::as_str)
                .context("last event has no run_id")?
                .to_owned();
            let previous_hash = last
                .get("event_hash")
                .and_then(Value::as_str)
                .context("last event has no event_hash")?
                .to_owned();
            let event = build_event(
                run_id,
                events.len() as u64 + 1,
                event_type,
                action,
                resource,
                Some(previous_hash),
                subject,
                None,
            )?;
            append_jsonl(&run, &event)?;
            println!("event appended to {}", run.display());
            Ok(())
        }
        EventCommand::Sign { file, key, out } => {
            let mut value = read_json_file(&file)?;
            sign_value(&mut value, &read_key(&key)?)?;
            write_pretty_json(&out, &value)?;
            println!("signed event written to {}", out.display());
            Ok(())
        }
        EventCommand::VerifySignature { file } => {
            verify_signature(&read_json_file(&file)?)?;
            println!("ok: event signature verifies");
            Ok(())
        }
    }
}

fn handle_key(command: KeyCommand) -> Result<()> {
    match command {
        KeyCommand::Generate { out } => {
            let signing_key = SigningKey::generate(&mut OsRng);
            let verifying_key = signing_key.verifying_key();
            let key = LocalKeyFile {
                schema: "agentprov.dev/local-key/v1".to_owned(),
                algorithm: "ed25519".to_owned(),
                key_id: format!("key_{}", Uuid::new_v4().simple()),
                public_key: hex::encode(verifying_key.to_bytes()),
                secret_key: hex::encode(signing_key.to_bytes()),
            };
            write_pretty_json(&out, &serde_json::to_value(key)?)?;
            println!("key written to {}", out.display());
            Ok(())
        }
    }
}

fn handle_policy(command: PolicyCommand) -> Result<()> {
    match command {
        PolicyCommand::Check {
            policy,
            agent,
            action,
            resource,
        } => {
            let policy = read_json_file(&policy)?;
            let decision = policy_decision(&policy, &agent, &action, &resource);
            print_json(&decision)
        }
    }
}

fn handle_demo(command: DemoCommand) -> Result<()> {
    match command {
        DemoCommand::ManualToolRun { out } => {
            fs::create_dir_all(&out).with_context(|| format!("create {}", out.display()))?;
            let run = out.join("run.jsonl");
            let run_id = "run_demo_manual_tool".to_owned();
            let mut events = Vec::new();
            events.push(build_event(
                run_id.clone(),
                1,
                "run.start".to_owned(),
                Some("trigger.manual".to_owned()),
                Some("demo".to_owned()),
                None,
                Some("research-agent".to_owned()),
                Some(json!({
                    "agent": "research-agent",
                    "agent_version": "0.1.0",
                    "actor_chain": ["danjdewhurst", "hermes", "research-agent"],
                })),
            )?);
            let previous = event_hash(events.last().unwrap())?;
            events.push(build_event(
                run_id.clone(),
                2,
                "llm.call".to_owned(),
                Some("model.invoke".to_owned()),
                Some("openai:gpt-example".to_owned()),
                Some(previous),
                Some("research-agent".to_owned()),
                Some(json!({"prompt_digest": "blake3:demo", "capture": "digest-only"})),
            )?);
            let previous = event_hash(events.last().unwrap())?;
            events.push(build_event(
                run_id.clone(),
                3,
                "permission.check".to_owned(),
                Some("discord.message.create".to_owned()),
                Some("discord://guild/demo/channel/demo".to_owned()),
                Some(previous),
                Some("research-agent".to_owned()),
                Some(json!({"decision": "allow", "policy_id": "policy_demo"})),
            )?);
            let previous = event_hash(events.last().unwrap())?;
            events.push(build_event(
                run_id,
                4,
                "tool.execute".to_owned(),
                Some("discord.message.create".to_owned()),
                Some("discord://guild/demo/channel/demo".to_owned()),
                Some(previous),
                Some("research-agent".to_owned()),
                Some(
                    json!({"tool": "discord.send_message", "output_digest": "blake3:demo-output"}),
                ),
            )?);
            write_jsonl(&run, &events)?;
            println!("demo written to {}", out.display());
            println!("Run verifies");
            println!("Agent: research-agent v0.1.0");
            println!("Trigger: manual");
            println!("Actor chain: danjdewhurst -> hermes -> research-agent");
            println!("Events: 4");
            println!("Permission checks: 1 allowed");
            println!("Tool calls: 1");
            println!("Event chain: valid");
            println!("Signatures: not present");
            Ok(())
        }
    }
}

fn handle_export(command: ExportCommand) -> Result<()> {
    match command {
        ExportCommand::Otel { file, out } => {
            let events = read_jsonl(&file)?;
            let spans: Vec<Value> = events.iter().map(to_otel_span).collect();
            write_pretty_json(
                &out,
                &json!({"resourceSpans": [{"scopeSpans": [{"spans": spans}]}]}),
            )?;
            println!("OTel-shaped export written to {}", out.display());
            Ok(())
        }
        ExportCommand::Openinference { file, out } => {
            let events = read_jsonl(&file)?;
            let spans: Vec<Value> = events.iter().map(to_openinference_span).collect();
            write_pretty_json(&out, &json!({"spans": spans}))?;
            println!("OpenInference-shaped export written to {}", out.display());
            Ok(())
        }
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

#[allow(clippy::too_many_arguments)]
fn build_event(
    run_id: String,
    sequence: u64,
    event_type: String,
    action: Option<String>,
    resource: Option<String>,
    previous_event_hash: Option<String>,
    subject: Option<String>,
    metadata: Option<Value>,
) -> Result<Value> {
    let mut event = json!({
        "schema": "agentprov.dev/event/v1",
        "event_id": format!("evt_{}", Uuid::new_v4().simple()),
        "run_id": run_id,
        "sequence": sequence,
        "timestamp": Utc::now(),
        "event_type": event_type,
        "subject": {"type": "agent", "id": subject.unwrap_or_else(|| "agent_01hxexample".to_owned())},
        "action": action,
        "resource": resource,
        "payload_digest": null,
        "previous_event_hash": previous_event_hash,
        "event_hash": null,
        "signature": null,
        "key_id": null,
        "metadata": metadata,
    });
    let hash = event_hash(&event)?;
    event["event_hash"] = Value::String(hash);
    Ok(event)
}

fn read_json_file(path: &Path) -> Result<Value> {
    let content =
        fs::read_to_string(path).with_context(|| format!("read JSON file {}", path.display()))?;
    serde_json::from_str(&content).with_context(|| format!("parse JSON file {}", path.display()))
}

fn write_pretty_json(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    fs::write(path, format!("{}\n", serde_json::to_string_pretty(value)?))
        .with_context(|| format!("write {}", path.display()))
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

fn event_hash(value: &Value) -> Result<String> {
    let mut unsigned = value.clone();
    remove_field(&mut unsigned, "event_hash");
    remove_field(&mut unsigned, "signature");
    canonical_hash(&unsigned)
}

fn verify_event_hash(value: &Value) -> Result<()> {
    let expected = value
        .get("event_hash")
        .and_then(Value::as_str)
        .context("event_hash must be present and a string")?;
    let actual = event_hash(value)?;
    if expected == actual {
        Ok(())
    } else {
        bail!("event hash mismatch: expected {expected}, actual {actual}")
    }
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

fn read_jsonl(path: &Path) -> Result<Vec<Value>> {
    let file = File::open(path).with_context(|| format!("open {}", path.display()))?;
    let mut values = Vec::new();
    for (index, line) in BufReader::new(file).lines().enumerate() {
        let line = line.with_context(|| format!("read line {}", index + 1))?;
        if line.trim().is_empty() {
            continue;
        }
        values.push(
            serde_json::from_str(&line)
                .with_context(|| format!("parse JSON line {} in {}", index + 1, path.display()))?,
        );
    }
    Ok(values)
}

fn write_jsonl(path: &Path, values: &[Value]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    for value in values {
        writeln!(file, "{}", serde_json::to_string(value)?)?;
    }
    Ok(())
}

fn append_jsonl(path: &Path, value: &Value) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .with_context(|| format!("open {} for append", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

struct VerifyReport {
    events: usize,
    signatures_present: bool,
}

fn verify_run_log(path: &Path, require_signatures: bool) -> Result<VerifyReport> {
    let events = read_jsonl(path)?;
    if events.is_empty() {
        bail!("run log contains no events");
    }
    let mut previous_hash: Option<String> = None;
    let mut signatures_present = false;
    for (index, event) in events.iter().enumerate() {
        let expected_sequence = index as u64 + 1;
        let sequence = event
            .get("sequence")
            .and_then(Value::as_u64)
            .context("event sequence must be an integer")?;
        if sequence != expected_sequence {
            bail!("event sequence mismatch: expected {expected_sequence}, actual {sequence}");
        }
        verify_event_hash(event)?;
        let actual_previous = event.get("previous_event_hash").and_then(Value::as_str);
        if actual_previous != previous_hash.as_deref() {
            bail!(
                "previous_event_hash mismatch at sequence {sequence}: expected {:?}, actual {:?}",
                previous_hash,
                actual_previous
            );
        }
        let signature = event.get("signature").filter(|value| !value.is_null());
        if signature.is_some() {
            verify_signature(event)?;
            signatures_present = true;
        } else if require_signatures {
            bail!("missing signature at sequence {sequence}");
        }
        previous_hash = event
            .get("event_hash")
            .and_then(Value::as_str)
            .map(str::to_owned);
    }
    Ok(VerifyReport {
        events: events.len(),
        signatures_present,
    })
}

fn read_key(path: &Path) -> Result<LocalKeyFile> {
    let value = read_json_file(path)?;
    serde_json::from_value(value).with_context(|| format!("parse key file {}", path.display()))
}

fn signing_key(key: &LocalKeyFile) -> Result<SigningKey> {
    let bytes = hex::decode(&key.secret_key).context("decode secret key hex")?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("secret key must be 32 bytes"))?;
    Ok(SigningKey::from_bytes(&bytes))
}

fn verifying_key(public_key: &str) -> Result<VerifyingKey> {
    let bytes = hex::decode(public_key).context("decode public key hex")?;
    let bytes: [u8; 32] = bytes
        .try_into()
        .map_err(|_| anyhow::anyhow!("public key must be 32 bytes"))?;
    VerifyingKey::from_bytes(&bytes).context("parse verifying key")
}

fn sign_value(value: &mut Value, key: &LocalKeyFile) -> Result<()> {
    remove_field(value, "signature");
    value["key_id"] = Value::String(key.key_id.clone());
    let hash = if value.get("event_hash").is_some() {
        event_hash(value)?
    } else {
        canonical_hash(value)?
    };
    if value.get("event_hash").is_some() {
        value["event_hash"] = Value::String(hash.clone());
    }
    let signature = signing_key(key)?.sign(hash.as_bytes());
    value["signature"] = json!({
        "algorithm": "ed25519",
        "key_id": key.key_id,
        "public_key": key.public_key,
        "signature": hex::encode(signature.to_bytes()),
        "signed_hash": hash,
    });
    Ok(())
}

fn verify_signature(value: &Value) -> Result<()> {
    let signature_value = value
        .get("signature")
        .filter(|value| !value.is_null())
        .context("signature must be present")?;
    let public_key = signature_value
        .get("public_key")
        .and_then(Value::as_str)
        .context("signature.public_key must be present")?;
    let signature_hex = signature_value
        .get("signature")
        .and_then(Value::as_str)
        .context("signature.signature must be present")?;
    let signed_hash = signature_value
        .get("signed_hash")
        .and_then(Value::as_str)
        .context("signature.signed_hash must be present")?;
    let actual_hash = if value.get("event_hash").is_some() {
        event_hash(value)?
    } else {
        let mut unsigned = value.clone();
        remove_field(&mut unsigned, "signature");
        canonical_hash(&unsigned)?
    };
    if signed_hash != actual_hash {
        bail!("signed hash mismatch: expected {signed_hash}, actual {actual_hash}");
    }
    let signature_bytes = hex::decode(signature_hex).context("decode signature hex")?;
    let signature = Signature::from_slice(&signature_bytes).context("parse signature")?;
    verifying_key(public_key)?.verify(signed_hash.as_bytes(), &signature)?;
    Ok(())
}

fn policy_decision(policy: &Value, agent: &str, action: &str, resource: &str) -> Value {
    if policy
        .get("agent_id")
        .and_then(Value::as_str)
        .is_some_and(|id| id != agent)
    {
        return decision(policy, "deny", "agent did not match policy");
    }
    if matches_rule(policy.get("deny"), action, resource) {
        return decision(policy, "deny", "matched deny rule");
    }
    if matches_rule(policy.get("require_approval"), action, resource) {
        return decision(policy, "require_approval", "matched require_approval rule");
    }
    if matches_rule(policy.get("allow"), action, resource) {
        return decision(policy, "allow", "matched allow rule");
    }
    decision(policy, "deny", "no matching allow rule")
}

fn decision(policy: &Value, decision: &str, reason: &str) -> Value {
    json!({
        "decision": decision,
        "policy_id": policy.get("policy_id").and_then(Value::as_str),
        "policy_version": policy.get("version").and_then(Value::as_str),
        "reason": reason,
    })
}

fn matches_rule(rules: Option<&Value>, action: &str, resource: &str) -> bool {
    rules.and_then(Value::as_array).is_some_and(|rules| {
        rules
            .iter()
            .any(|rule| rule_matches(rule, action, resource))
    })
}

fn rule_matches(rule: &Value, action: &str, resource: &str) -> bool {
    let rule_action = rule.get("action").and_then(Value::as_str).unwrap_or("");
    let rule_resource = rule.get("resource").and_then(Value::as_str).unwrap_or("");
    pattern_matches(rule_action, action) && pattern_matches(rule_resource, resource)
}

fn pattern_matches(pattern: &str, value: &str) -> bool {
    pattern == "*"
        || pattern == value
        || pattern
            .strip_suffix('*')
            .is_some_and(|prefix| value.starts_with(prefix))
}

fn to_otel_span(event: &Value) -> Value {
    let event_type = event
        .get("event_type")
        .and_then(Value::as_str)
        .unwrap_or("event");
    json!({
        "traceId": event.get("run_id"),
        "spanId": event.get("event_id"),
        "name": event_type,
        "attributes": {
            "agentprov.event.type": event_type,
            "agentprov.event.hash": event.get("event_hash"),
            "agentprov.event.previous_hash": event.get("previous_event_hash"),
            "agentprov.event.sequence": event.get("sequence"),
            "gen_ai.operation.name": otel_operation_name(event_type),
            "gen_ai.agent.id": event.pointer("/subject/id"),
            "gen_ai.tool.name": event.get("resource"),
        }
    })
}

fn to_openinference_span(event: &Value) -> Value {
    let event_type = event
        .get("event_type")
        .and_then(Value::as_str)
        .unwrap_or("event");
    json!({
        "trace_id": event.get("run_id"),
        "span_id": event.get("event_id"),
        "name": event_type,
        "attributes": {
            "openinference.span.kind": openinference_kind(event_type),
            "input.value": event.get("action"),
            "metadata": event.get("metadata"),
            "agent.name": event.pointer("/subject/id"),
            "tool.name": event.get("resource"),
            "agentprov.event.hash": event.get("event_hash"),
        }
    })
}

fn otel_operation_name(event_type: &str) -> &str {
    match event_type {
        "run.start" | "agent.invoke" => "invoke_agent",
        "agent.plan" => "plan",
        "tool.execute" => "execute_tool",
        "memory.read" => "search_memory",
        "memory.write" => "update_memory",
        _ => event_type,
    }
}

fn openinference_kind(event_type: &str) -> &str {
    match event_type {
        "llm.call" => "LLM",
        "tool.execute" => "TOOL",
        "prompt.render" => "PROMPT",
        "agent.invoke" | "agent.plan" | "run.start" => "AGENT",
        "retrieval.search" => "RETRIEVER",
        "guardrail.check" => "GUARDRAIL",
        "eval.run" => "EVALUATOR",
        _ => "CHAIN",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_hash_is_independent_of_object_key_order() {
        let a = json!({"b": 2, "a": 1, "nested": {"z": true, "c": null}});
        let b = json!({"nested": {"c": null, "z": true}, "a": 1, "b": 2});
        assert_eq!(canonical_hash(&a).unwrap(), canonical_hash(&b).unwrap());
    }

    #[test]
    fn event_hash_excludes_event_hash_and_signature_fields() {
        let mut event = json!({"event_hash": "ignored", "signature": "ignored", "sequence": 1});
        remove_field(&mut event, "event_hash");
        remove_field(&mut event, "signature");
        assert_eq!(event, json!({"sequence": 1}));
    }

    #[test]
    fn prefix_wildcard_policy_rule_matches() {
        assert!(pattern_matches(
            "discord://guild/123/*",
            "discord://guild/123/channel/456"
        ));
        assert!(!pattern_matches(
            "discord://guild/123/*",
            "discord://guild/999/channel/456"
        ));
    }
}
