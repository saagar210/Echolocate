use std::process::Command;
use std::time::Duration;

/// Attempt to resolve a hostname for an IP address.
/// Tries reverse DNS first, then system hostname lookups.
pub async fn resolve_hostname(ip: &str) -> Option<String> {
    let ip = ip.to_string();

    // Run in a blocking thread since it shells out
    let result = tokio::task::spawn_blocking(move || {
        resolve_hostname_sync(&ip)
    })
    .await
    .ok()?;

    result
}

/// Synchronous hostname resolution using system tools.
fn resolve_hostname_sync(ip: &str) -> Option<String> {
    // Try `host` command for reverse DNS
    let output = Command::new("host")
        .arg(ip)
        .output()
        .ok()?;

    if output.status.success() {
        let text = String::from_utf8_lossy(&output.stdout);
        // Output format: "1.168.192.in-addr.arpa domain name pointer hostname.local."
        if let Some(ptr_line) = text.lines().find(|l| l.contains("domain name pointer")) {
            let hostname = ptr_line
                .split("domain name pointer")
                .nth(1)?
                .trim()
                .trim_end_matches('.');
            if !hostname.is_empty() {
                return Some(hostname.to_string());
            }
        }
    }

    None
}

/// Resolve hostnames for multiple IPs concurrently.
pub async fn resolve_hostnames(ips: &[String]) -> Vec<(String, Option<String>)> {
    let mut handles = Vec::new();

    for ip in ips {
        let ip = ip.clone();
        let handle = tokio::spawn(async move {
            let hostname = tokio::time::timeout(
                Duration::from_secs(2),
                resolve_hostname(&ip),
            )
            .await
            .ok()
            .flatten();

            (ip, hostname)
        });
        handles.push(handle);
    }

    let mut results = Vec::new();
    for handle in handles {
        if let Ok(result) = handle.await {
            results.push(result);
        }
    }

    results
}
