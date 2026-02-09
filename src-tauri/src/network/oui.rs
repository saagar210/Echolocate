use std::collections::HashMap;

/// MAC address vendor lookup using the IEEE OUI database.
/// Maps the first 3 bytes of a MAC address to an organization name.
pub struct OuiDatabase {
    entries: HashMap<[u8; 3], String>,
}

impl OuiDatabase {
    /// Load the OUI database from the bundled CSV resource.
    pub fn load(app: &tauri::AppHandle) -> Result<Self, Box<dyn std::error::Error>> {
        use tauri::Manager;
        let resource_path = app
            .path()
            .resource_dir()
            .map_err(|e| format!("Failed to get resource dir: {}", e))?
            .join("resources")
            .join("oui.csv");

        if !resource_path.exists() {
            log::warn!("OUI database not found at {}", resource_path.display());
            return Ok(Self::empty());
        }

        let mut entries = HashMap::new();
        let mut reader = csv::ReaderBuilder::new()
            .has_headers(true)
            .from_path(&resource_path)?;

        for result in reader.records() {
            let record = match result {
                Ok(r) => r,
                Err(e) => {
                    log::debug!("Skipping malformed OUI record: {}", e);
                    continue;
                }
            };

            // IEEE CSV format: "Registry","Assignment","Organization Name",...
            // Assignment is the 6-char hex OUI (e.g., "AABBCC")
            if record.len() < 3 {
                continue;
            }

            let assignment = &record[1];
            let org_name = &record[2];

            if let Some(prefix) = parse_oui_hex(assignment) {
                entries.insert(prefix, org_name.to_string());
            }
        }

        log::info!("Loaded {} OUI entries", entries.len());
        Ok(Self { entries })
    }

    /// Create an empty OUI database (fallback when file is missing).
    pub fn empty() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    /// Look up the vendor for a MAC address.
    /// Accepts formats: "AA:BB:CC:DD:EE:FF", "AA-BB-CC-DD-EE-FF", "AABB.CCDD.EEFF"
    pub fn lookup(&self, mac: &str) -> Option<&str> {
        let prefix = parse_mac_prefix(mac)?;
        self.entries.get(&prefix).map(|s| s.as_str())
    }

    /// Number of entries loaded.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// Parse a 6-char hex string (e.g., "AABBCC") into 3 bytes.
fn parse_oui_hex(hex: &str) -> Option<[u8; 3]> {
    let hex = hex.trim();
    if hex.len() != 6 {
        return None;
    }
    let bytes = hex::decode(hex).ok()?;
    if bytes.len() != 3 {
        return None;
    }
    Some([bytes[0], bytes[1], bytes[2]])
}

/// Extract the OUI prefix (first 3 bytes) from a MAC address string.
fn parse_mac_prefix(mac: &str) -> Option<[u8; 3]> {
    let cleaned: String = mac
        .chars()
        .filter(|c| c.is_ascii_hexdigit())
        .collect();

    if cleaned.len() < 6 {
        return None;
    }

    let bytes = hex::decode(&cleaned[..6]).ok()?;
    Some([bytes[0], bytes[1], bytes[2]])
}

/// Simple hex decode (we avoid adding the `hex` crate for just this)
mod hex {
    pub fn decode(hex: &str) -> Result<Vec<u8>, ()> {
        if hex.len() % 2 != 0 {
            return Err(());
        }
        (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).map_err(|_| ()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mac_prefix_colon() {
        let prefix = parse_mac_prefix("AA:BB:CC:DD:EE:FF").unwrap();
        assert_eq!(prefix, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_parse_mac_prefix_dash() {
        let prefix = parse_mac_prefix("AA-BB-CC-DD-EE-FF").unwrap();
        assert_eq!(prefix, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_parse_mac_prefix_dot() {
        let prefix = parse_mac_prefix("AABB.CCDD.EEFF").unwrap();
        assert_eq!(prefix, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_parse_mac_prefix_lowercase() {
        let prefix = parse_mac_prefix("aa:bb:cc:dd:ee:ff").unwrap();
        assert_eq!(prefix, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_parse_mac_prefix_too_short() {
        assert!(parse_mac_prefix("AA:BB").is_none());
    }

    #[test]
    fn test_parse_oui_hex() {
        let prefix = parse_oui_hex("AABBCC").unwrap();
        assert_eq!(prefix, [0xAA, 0xBB, 0xCC]);
    }

    #[test]
    fn test_empty_db_lookup() {
        let db = OuiDatabase::empty();
        assert!(db.lookup("AA:BB:CC:DD:EE:FF").is_none());
    }

    #[test]
    fn test_manual_lookup() {
        let mut entries = HashMap::new();
        entries.insert([0xDC, 0xA6, 0x32], "Raspberry Pi Trading Ltd".to_string());
        entries.insert([0x3C, 0x22, 0xFB], "Apple, Inc.".to_string());

        let db = OuiDatabase { entries };

        assert_eq!(db.lookup("DC:A6:32:AA:BB:CC"), Some("Raspberry Pi Trading Ltd"));
        assert_eq!(db.lookup("3C:22:FB:00:11:22"), Some("Apple, Inc."));
        assert!(db.lookup("00:00:00:00:00:00").is_none());
    }
}
