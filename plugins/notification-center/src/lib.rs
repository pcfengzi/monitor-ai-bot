// plugins/notification-center/src/lib.rs
mod db;
mod router;
mod template;
mod risk_guard;
mod channel;
mod types;
mod metrics_bridge;

use std::os::raw::c_char;
use std::thread;

use once_cell::sync::OnceCell;
use plugin_api::{PluginContext, PluginMeta, LogLevel};

use crate::metrics_bridge::HostBridge;

// 给其它模块用的：全局 HostBridge
static HOST_BRIDGE: OnceCell<HostBridge> = OnceCell::new();

// 你之前的 meta 保持风格一致即可
static NAME: &[u8] = b"notification-center\0";
static VERSION: &[u8] = b"0.2.0\0";
static KIND: &[u8] = b"notification\0";

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: NAME.as_ptr() as *const c_char,
        version: VERSION.as_ptr() as *const c_char,
        kind: KIND.as_ptr() as *const c_char,
    }
}

// 如果你的框架里有 plugin_api_info，这里可以继续保持原来的实现
// 这里省略，只展示 run_with_ctx

#[unsafe(no_mangle)]
pub extern "C" fn run() {
    // 兼容老的 host，简单输出一下
    println!("[notification-center] run() called without context (legacy)");
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };

    // 初始化 HostBridge，让插件内部任意地方都能记日志 / 上报 metric
    let bridge = HostBridge::from_ctx(ctx, "notification-center".to_string());
    let _ = HOST_BRIDGE.set(bridge);

    HostBridge::log_static(LogLevel::Info, "[notification-center] run_with_ctx");

    // 起一个线程跑 tokio runtime（避免阻塞 host 主线程）
    thread::spawn(|| {
        let rt = tokio::runtime::Runtime::new().expect("create runtime failed");
        rt.block_on(async {
            // 1) 初始化自己的 DB 表（不会修改 storage）
            if let Err(e) = db::init_db().await {
                HostBridge::log_static(
                    LogLevel::Error,
                    &format!("[notification-center] init_db error: {e}"),
                );
                return;
            }

            // 2) 启动 HTTP 服务 + 异步发送队列 worker
            if let Err(e) = router::start_server().await {
                HostBridge::log_static(
                    LogLevel::Error,
                    &format!("[notification-center] http server error: {e}"),
                );
            }
        });
    });

    HostBridge::log_static(
        LogLevel::Info,
        "[notification-center] background server spawned",
    );
}

// 其它模块如果想用 HostBridge：直接 use crate::HOST_BRIDGE;
pub fn host_bridge() -> Option<&'static HostBridge> {
    HOST_BRIDGE.get()
}
