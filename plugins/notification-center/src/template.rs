// plugins/notification-center/src/template.rs
use regex::Regex;
use serde_json::Value;

/// 简单的 {{var}} 替换
pub fn render(content: &str, vars: &Value) -> String {
    lazy_static::lazy_static! {
        static ref RE: Regex = Regex::new(r"\{\{\s*([a-zA-Z0-9_.]+)\s*\}\}").unwrap();
    }

    RE.replace_all(content, |caps: &regex::Captures| {
        let key = &caps[1];
        lookup(vars, key).unwrap_or_else(|| "".to_string())
    })
    .into_owned()
}

fn lookup(vars: &Value, path: &str) -> Option<String> {
    let mut cur = vars;
    for part in path.split('.') {
        cur = match cur.get(part) {
            Some(v) => v,
            None => return None,
        };
    }
    match cur {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    }
}
