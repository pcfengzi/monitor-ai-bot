// plugins/notification-center/src/metrics_bridge.rs
use std::ffi::CString;
use std::os::raw::c_char;

use chrono::Utc;
use plugin_api::{LogLevel, MetricSample, PluginContext};

#[derive(Clone)]
pub struct HostBridge {
    pub plugin_name: String,
    log_fn: unsafe extern "C" fn(LogLevel, *const c_char),
    emit_metric_fn: unsafe extern "C" fn(MetricSample),
}

impl HostBridge {
    pub fn from_ctx(ctx: &PluginContext, plugin_name: String) -> Self {
        Self {
            plugin_name,
            log_fn: ctx.log_fn,
            emit_metric_fn: ctx.emit_metric_fn,
        }
    }

    pub fn log(&self, level: LogLevel, msg: &str) {
        let full = format!("[{}] {}", self.plugin_name, msg);
        let c = CString::new(full).unwrap_or_else(|_| CString::new("log error").unwrap());
        unsafe { (self.log_fn)(level, c.as_ptr()) }
    }

    pub fn metric(&self, name: &str, value: f64) {
        let now = Utc::now()
            .timestamp_millis();

        let cname = CString::new(name).unwrap_or_else(|_| CString::new("metric_error").unwrap());

        let sample = MetricSample {
            name: cname.as_ptr(),
            value,
            timestamp_ms: now,
        };

        unsafe {
            (self.emit_metric_fn)(sample);
        }
    }

    // 静态方法，方便在任何地方调用（通过 crate::host_bridge()）
    pub fn log_static(level: LogLevel, msg: &str) {
        if let Some(b) = crate::host_bridge() {
            b.log(level, msg);
        }
    }

    pub fn metric_static(name: &str, value: f64) {
        if let Some(b) = crate::host_bridge() {
            b.metric(name, value);
        }
    }
}
