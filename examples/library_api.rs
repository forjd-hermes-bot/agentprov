use agentprov::{AppendEventInput, EventInput, append_event_to_run, init_run_log, verify_run_log};
use serde_json::json;
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    let out = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("runs/library-api.jsonl"));

    let start = EventInput::new("run_library_api_example", 1, "run.start")
        .action("trigger.api")
        .resource("example://library-api")
        .subject("agent_01hxexample")
        .metadata(json!({
            "agent": "research-agent",
            "integration": "rust-library",
            "capture": "digest-only"
        }));
    init_run_log(&out, start)?;

    let tool = AppendEventInput::new("tool.execute")
        .action("example.lookup")
        .resource("example://dataset/customer-summary")
        .subject("agent_01hxexample")
        .metadata(json!({
            "result_digest": "blake3:example-result",
            "redaction": "payload omitted"
        }));
    append_event_to_run(&out, tool)?;

    verify_run_log(&out, false)?;
    println!("Library API run log written to {}", out.display());
    Ok(())
}
