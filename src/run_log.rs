use crate::event::verify_event_hash;
use crate::signing::verify_signature;
use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

pub struct VerifyReport {
    pub events: usize,
    pub signatures_present: bool,
}

pub fn read_jsonl(path: &Path) -> Result<Vec<Value>> {
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

pub fn write_jsonl(path: &Path, values: &[Value]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    let mut file = File::create(path).with_context(|| format!("create {}", path.display()))?;
    for value in values {
        writeln!(file, "{}", serde_json::to_string(value)?)?;
    }
    Ok(())
}

pub fn append_jsonl(path: &Path, value: &Value) -> Result<()> {
    let mut file = OpenOptions::new()
        .append(true)
        .open(path)
        .with_context(|| format!("open {} for append", path.display()))?;
    writeln!(file, "{}", serde_json::to_string(value)?)?;
    Ok(())
}

pub fn verify_run_log(path: &Path, require_signatures: bool) -> Result<VerifyReport> {
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
