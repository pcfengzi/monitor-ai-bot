use plugin_api::{PluginContext, PluginMeta};
use std::os::raw::c_char;
use std::ffi::CString;
use chrono::Utc;

// 静态元信息
static NAME: &str = "agent-aggregator";
static VERSION: &str = "0.1.0";
static KIND: &str = "agent";

fn c_string(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: c_string(NAME),
        version: c_string(VERSION),
        kind: c_string(KIND),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };

    // 暂时只做一件事：记录一条“我还活着”的 log
    let now = Utc::now().to_rfc3339();
    let msg = format!("[agent-aggregator] 执行一次聚合任务 at {}", now);
    ctx.log_info(&msg);

    // TODO：
    // 后面这里会：
    // - 通过 storage 或 api-server 查询最近的 agent_* metric
    // - 计算各 agent 健康状态
    // - 对超阈值的情况 emit_metric / log / alert
}
