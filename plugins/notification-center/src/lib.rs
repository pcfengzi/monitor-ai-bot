use plugin_api::{PluginMeta, PluginContext, PluginApiInfo};
use std::os::raw::c_char;
use std::ffi::CString;
use std::sync::OnceLock;
mod db;
mod router;
mod template;
mod risk_guard;
mod channel;

use db::init_db;


mod router;
mod template;
mod risk_guard;
mod channel;

static START_API_ONCE: OnceLock<()> = OnceLock::new();

const PLUGIN_NAME: &str = "notification-center";
const VERSION: &str = "0.1.0";
const KIND: &str = "notification";

const API_PORT: u16 = 5601;
const API_PREFIX: &str = "/";

fn cstr(s: &str) -> *const c_char {
    CString::new(s).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn meta() -> PluginMeta {
    PluginMeta {
        name: cstr(PLUGIN_NAME),
        version: cstr(VERSION),
        kind: cstr(KIND),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn plugin_api_info() -> PluginApiInfo {
    PluginApiInfo {
        port: API_PORT,
        prefix: cstr(API_PREFIX),
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn run_with_ctx(ctx: *mut PluginContext) {
    if ctx.is_null() {
        return;
    }
    let ctx = unsafe { &mut *ctx };

    ctx.log_info("[notification-center] run_with_ctx triggered");

    START_API_ONCE.get_or_init(|| {
        std::thread::spawn(|| {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                // ⭐ 初始化通知插件自己的 DB & 表
                let _db = init_db().await;

                crate::router::start_server(API_PORT).await;
            });
        });
    });

    ctx.log_info("[notification-center] API server running");
}
