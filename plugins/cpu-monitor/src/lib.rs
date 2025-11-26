use std::os::raw::c_char;
use std::ffi::CString;

use plugin_api::{
    LogLevel,
    MetricSample,
    PluginContext,
    PluginMeta,
};

static PLUGIN_NAME: &[u8] = b"cpu-monitor\0";
static PLUGIN_VERSION: &[u8] = b"0.2.0\0";
static PLUGIN_KIND: &[u8] = b"cpu\0";

/// 旧接口：无上下文，host 仍然可以调用
#[unsafe(no_mangle)]
pub extern "C" fn run() {
    println!("[cpu-monitor] run() 被调用（无上下文版本）");
}

/// 新接口：带上下文
#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    // 安全起见先检查指针
    if ctx.is_null() {
        // 退而求其次，打印一下
        println!("[cpu-monitor] run_with_ctx 收到空 ctx 指针");
        return;
    }

    // 安全块内操作
    unsafe {
        let ctx = &*ctx;

        // 1. 通过 log_fn 写一条日志
        let msg = CString::new("CPU 插件开始执行").unwrap();
        (ctx.log_fn)(LogLevel::Info, msg.as_ptr());

        // 2. 模拟采集一个 CPU 使用率指标
        let metric_name = CString::new("cpu_usage").unwrap();
        let sample = MetricSample {
            name: metric_name.as_ptr(),
            value: 42.0, // 假数据，后面你可以接 sysinfo / heim 等 crate
            timestamp_ms: current_timestamp_ms(),
        };

        (ctx.emit_metric_fn)(sample);

        // 3. 再打一条日志
        let done_msg = CString::new("CPU 插件执行完毕").unwrap();
        (ctx.log_fn)(LogLevel::Debug, done_msg.as_ptr());
    }
}

/// 元信息函数保持不变
#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: PLUGIN_NAME.as_ptr() as *const c_char,
        version: PLUGIN_VERSION.as_ptr() as *const c_char,
        kind: PLUGIN_KIND.as_ptr() as *const c_char,
    }
}

fn current_timestamp_ms() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    now.as_millis() as i64
}
