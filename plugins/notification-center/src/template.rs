use lazy_static::lazy_static;
use regex::Regex;
use serde_json::Value;

lazy_static! {
    static ref TEMPLATE_MAP: std::collections::HashMap<&'static str, &'static str> = {
        let mut map = std::collections::HashMap::new();
        map.insert("order_payed", "您的订单 {{order_id}} 已支付成功，金额 ¥{{amount}}。");
        map.insert("welcome", "欢迎 {{name}}，感谢您的注册！");
        map
    };
}

pub fn render_template(scene: &str, vars: &Value) -> String {
    let tpl = TEMPLATE_MAP.get(scene).unwrap_or(&"");

    let mut s = tpl.to_string();

    let re = Regex::new(r"\{\{(\w+)\}\}").unwrap();
    for cap in re.captures_iter(tpl) {
        let key = &cap[1];
        if let Some(v) = vars.get(key) {
            s = s.replace(&format!("{{{{{key}}}}}"), v.as_str().unwrap_or(""));
        }
    }

    s
}
