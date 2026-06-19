use assert_cmd::Command;
use predicates::prelude::*;
use serde_json::Value;
use std::fs;
use tempfile::tempdir;

#[test]
fn manifest_example_prints_valid_json() {
    let output = Command::cargo_bin("agentprov")
        .unwrap()
        .args(["manifest", "example"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();

    let value: Value = serde_json::from_slice(&output).unwrap();
    assert_eq!(value["schema"], "agentprov.dev/manifest/v1");
    assert_eq!(value["owner"]["id"], "danjdewhurst");
}

#[test]
fn event_hash_outputs_blake3_digest() {
    let mut cmd = Command::cargo_bin("agentprov").unwrap();
    cmd.args(["event", "hash", "examples/event.json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("blake3:"));
}

#[test]
fn event_verify_accepts_matching_hash() {
    let hash_output = Command::cargo_bin("agentprov")
        .unwrap()
        .args(["event", "hash", "examples/event.json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let hash = String::from_utf8(hash_output).unwrap().trim().to_owned();

    let mut value: Value =
        serde_json::from_str(&fs::read_to_string("examples/event.json").unwrap()).unwrap();
    value["event_hash"] = Value::String(hash);

    let dir = tempdir().unwrap();
    let path = dir.path().join("event.json");
    fs::write(&path, serde_json::to_string_pretty(&value).unwrap()).unwrap();

    Command::cargo_bin("agentprov")
        .unwrap()
        .args(["event", "verify", path.to_str().unwrap()])
        .assert()
        .success()
        .stdout(predicate::str::contains("ok: event hash verifies"));
}
