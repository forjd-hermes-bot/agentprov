use serde_json::{Value, json};

pub fn policy_decision(policy: &Value, agent: &str, action: &str, resource: &str) -> Value {
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

pub fn pattern_matches(pattern: &str, value: &str) -> bool {
    pattern == "*"
        || pattern == value
        || pattern
            .strip_suffix('*')
            .is_some_and(|prefix| value.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::*;

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
