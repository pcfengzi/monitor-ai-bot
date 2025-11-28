use std::os::raw::{c_char, c_longlong};

/// 旧版：无上下文的运行函数
pub type PluginRunFunc = extern "C" fn();

/// 插件元信息
#[repr(C)]
pub struct PluginMeta {
    pub name: *const c_char,
    pub version: *const c_char,
    pub kind: *const c_char,  // 比如 "cpu" / "memory" / "network" / "ai"
}

/// 返回插件元信息的函数签名
pub type PluginMetaFunc = extern "C" fn() -> PluginMeta;

/// 日志级别（给插件用的 FFI 版）
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum LogLevel {
    Debug = 0,
    Info = 1,
    Warn = 2,
    Error = 3,
}

/// 简化版指标采样（给插件用的 FFI 版）
/// 时间用毫秒时间戳，host 负责转为 chrono::DateTime 等复杂类型。
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MetricSample {
    /// 指标名，如 "cpu_usage"、"memory_used"
    pub name: *const c_char,
    /// 数值
    pub value: f64,
    /// 时间戳（毫秒）
    pub timestamp_ms: c_longlong,
}

/// 插件可以通过这个上下文调用 host 提供的功能
#[repr(C)]
pub struct PluginContext {
    /// 主机版本（协议/ABI版本），后续做兼容判断用
    pub host_version: u32,

    /// 由 host 提供的日志函数：
    /// 插件调用时： (level, msg_c_str)
    pub log_fn: extern "C" fn(level: LogLevel, msg: *const c_char),

    /// 由 host 提供的指标上报函数：
    /// 插件调用时： emit_metric_fn(sample)
    pub emit_metric_fn: extern "C" fn(sample: MetricSample),
}

/// 新版：带上下文的运行函数签名
pub type PluginRunWithContextFunc = extern "C" fn(ctx: *mut PluginContext);



/// 插件对外暴露的 HTTP API 信息（可选）
#[repr(C)]
pub struct PluginApiInfo {
    /// 插件内部 HTTP server 监听的端口，例如 5501
    pub port: u16,
    /// 统一前缀，例如 "/api" 或 "/"
    /// 如果不需要前缀，用 "/" 即可
    pub prefix: *const c_char,
}

/// 插件可以（可选）导出这么一个函数：
///
/// #[unsafe(no_mangle)]
/// pub extern "C" fn plugin_api_info() -> PluginApiInfo { ... }
pub type PluginApiInfoFunc = unsafe extern "C" fn() -> PluginApiInfo;
