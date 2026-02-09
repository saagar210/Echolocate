use regex::Regex;
use std::process::Command;
use std::time::Duration;

/// Ping a single IP address and return the latency in milliseconds.
/// Uses the system `ping` command (no raw sockets needed).
pub async fn ping(ip: &str) -> Option<f64> {
    let ip = ip.to_string();

    tokio::task::spawn_blocking(move || ping_sync(&ip))
        .await
        .ok()?
}

/// Synchronous ping using system command.
fn ping_sync(ip: &str) -> Option<f64> {
    let output = Command::new("ping")
        .args(["-c", "1", "-W", "2", "-n", ip])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let text = String::from_utf8_lossy(&output.stdout);
    parse_ping_output(&text)
}

/// Extract RTT from ping output.
/// macOS format: "round-trip min/avg/max/stddev = 1.234/1.456/1.789/0.123 ms"
/// Also matches: "time=1.234 ms" in individual ping lines
fn parse_ping_output(output: &str) -> Option<f64> {
    // Try the per-line format first
    let time_re = Regex::new(r"time[=<](\d+\.?\d*)\s*ms").unwrap();
    if let Some(caps) = time_re.captures(output) {
        return caps[1].parse().ok();
    }

    // Try the summary format
    let rtt_re = Regex::new(r"min/avg/max/\w+ = [\d.]+/([\d.]+)/").unwrap();
    if let Some(caps) = rtt_re.captures(output) {
        return caps[1].parse().ok();
    }

    None
}

/// Ping multiple IPs concurrently, returning (ip, latency_ms) pairs.
pub async fn ping_sweep(ips: &[String], max_concurrent: usize) -> Vec<(String, Option<f64>)> {
    use tokio::sync::Semaphore;
    use std::sync::Arc;

    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let mut handles = Vec::new();

    for ip in ips {
        let ip = ip.clone();
        let sem = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();
            let latency = tokio::time::timeout(
                Duration::from_secs(3),
                ping(&ip),
            )
            .await
            .ok()
            .flatten();

            (ip, latency)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ping_time_equals() {
        let output = "64 bytes from 192.168.1.1: icmp_seq=0 ttl=64 time=1.234 ms";
        assert_eq!(parse_ping_output(output), Some(1.234));
    }

    #[test]
    fn test_parse_ping_time_less_than() {
        let output = "64 bytes from 192.168.1.1: icmp_seq=0 ttl=64 time<1 ms";
        // The regex won't match "<1" as a float, but we handle it
        // Actually "time<1" → captures "1"
        assert_eq!(parse_ping_output(output), Some(1.0));
    }

    #[test]
    fn test_parse_ping_summary() {
        let output = "round-trip min/avg/max/stddev = 1.234/2.567/3.890/0.456 ms";
        assert_eq!(parse_ping_output(output), Some(2.567));
    }

    #[test]
    fn test_parse_ping_no_response() {
        let output = "Request timeout for icmp_seq 0";
        assert_eq!(parse_ping_output(output), None);
    }
}
