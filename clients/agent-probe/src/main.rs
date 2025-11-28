use anyhow::Result;
use serde::Serialize;
use std::{env, time::Duration};
use sysinfo::{System, SystemExt, CpuExt};
use tokio::time::sleep;

#[derive(Serialize)]
struct AgentMetric {
    time: String,
    agent_id: String,
    host: String,
    cpu_usage: f64,
    memory_used: u64,
    memory_total: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let agent_id = env::var("AGENT_ID").unwrap_or_else(|_| "agent-unknown".to_string());
    let api_base = env::var("MONITOR_AI_API_BASE")
        .unwrap_or_else(|_| "http://127.0.0.1:3001".to_string());
    let interval_secs: u64 = env::var("AGENT_INTERVAL_SECS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);

    println!("Agent 启动: id={agent_id}, api_base={api_base}, interval={interval_secs}s");

    let client = reqwest::Client::new();

    loop {
        if let Err(e) = collect_and_send(&client, &api_base, &agent_id).await {
            eprintln!("[agent] 上报失败: {e}");
        }
        sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn collect_and_send(
    client: &reqwest::Client,
    api_base: &str,
    agent_id: &str,
) -> Result<()> {
    let mut sys = System::new_all();
    sys.refresh_all();

    let host = sys.host_name().unwrap_or_else(|| "unknown-host".to_string());

    let total_memory = sys.total_memory();     // KiB
    let used_memory = sys.used_memory();       // KiB

    let global_cpu = sys.global_cpu_info();
    let cpu_usage = global_cpu.cpu_usage() as f64; // 百分比

    let payload = AgentMetric {
        time: chrono::Utc::now().to_rfc3339(),
        agent_id: agent_id.to_string(),
        host,
        cpu_usage,
        memory_used: used_memory * 1024,
        memory_total: total_memory * 1024,
    };

    let url = format!("{}/agent/metrics", api_base);
    let res = client.post(url).json(&payload).send().await?;

    if !res.status().is_success() {
        let text = res.text().await.unwrap_or_default();
        anyhow::bail!("HTTP {}: {}", res.status(), text);
    }

    Ok(())
}
