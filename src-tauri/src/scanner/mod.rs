pub mod fingerprint;
pub mod orchestrator;
pub mod passive;
pub mod ping;
pub mod port;

use serde::{Deserialize, Serialize};

/// A device discovered during a scan (raw scan result before DB enrichment).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredDevice {
    pub ip: String,
    pub mac: Option<String>,
    pub hostname: Option<String>,
    pub is_gateway: bool,
}

/// Scan configuration passed from the frontend.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanConfig {
    pub interface_id: String,
    pub scan_type: ScanType,
    pub port_range: PortRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ScanType {
    Quick,
    Full,
    PortOnly,
    Passive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PortRange {
    Top100,
    Top1000,
    Custom(Vec<u16>),
}

/// Result of a completed scan.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanResult {
    pub scan_id: String,
    pub devices_found: u32,
    pub new_devices: u32,
    pub duration_ms: u64,
}
