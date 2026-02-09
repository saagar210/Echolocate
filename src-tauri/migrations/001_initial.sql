-- Echolocate database schema v1
-- All tables for device discovery, scanning, alerts, and settings

-- Track applied migrations
CREATE TABLE IF NOT EXISTS _migrations (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    applied_at TEXT DEFAULT (datetime('now'))
);

-- Network interfaces on this machine
CREATE TABLE interfaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    ip_address TEXT,
    subnet_mask TEXT,
    mac_address TEXT,
    gateway_ip TEXT,
    is_active BOOLEAN DEFAULT 1,
    last_updated TEXT DEFAULT (datetime('now'))
);

-- Discovered devices
CREATE TABLE devices (
    id TEXT PRIMARY KEY,
    mac_address TEXT UNIQUE,
    vendor TEXT,
    hostname TEXT,
    custom_name TEXT,
    device_type TEXT NOT NULL DEFAULT 'unknown',
    os_guess TEXT,
    os_confidence REAL DEFAULT 0.0,
    is_trusted BOOLEAN DEFAULT 0,
    is_gateway BOOLEAN DEFAULT 0,
    notes TEXT,
    first_seen TEXT DEFAULT (datetime('now')),
    last_seen TEXT DEFAULT (datetime('now')),
    created_at TEXT DEFAULT (datetime('now'))
);

-- IP addresses (devices can have multiple over time / DHCP changes)
CREATE TABLE device_ips (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    ip_address TEXT NOT NULL,
    is_current BOOLEAN DEFAULT 1,
    first_seen TEXT DEFAULT (datetime('now')),
    last_seen TEXT DEFAULT (datetime('now'))
);

-- Scan results
CREATE TABLE scans (
    id TEXT PRIMARY KEY,
    interface_id TEXT REFERENCES interfaces(id),
    scan_type TEXT NOT NULL,
    status TEXT DEFAULT 'running',
    devices_found INTEGER DEFAULT 0,
    new_devices INTEGER DEFAULT 0,
    duration_ms INTEGER,
    started_at TEXT DEFAULT (datetime('now')),
    completed_at TEXT
);

-- Open ports per device per scan
CREATE TABLE device_ports (
    id TEXT PRIMARY KEY,
    device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    scan_id TEXT REFERENCES scans(id) ON DELETE CASCADE,
    port INTEGER NOT NULL,
    protocol TEXT DEFAULT 'tcp',
    state TEXT DEFAULT 'open',
    service_name TEXT,
    banner TEXT,
    discovered_at TEXT DEFAULT (datetime('now'))
);

-- Latency history (for response time graphs)
CREATE TABLE latency_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    device_id TEXT NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    latency_ms REAL,
    measured_at TEXT DEFAULT (datetime('now'))
);

-- Alerts
CREATE TABLE alerts (
    id TEXT PRIMARY KEY,
    alert_type TEXT NOT NULL,
    device_id TEXT REFERENCES devices(id),
    message TEXT NOT NULL,
    severity TEXT DEFAULT 'info',
    is_read BOOLEAN DEFAULT 0,
    created_at TEXT DEFAULT (datetime('now'))
);

-- Alert rules (user-configurable)
CREATE TABLE alert_rules (
    id TEXT PRIMARY KEY,
    rule_type TEXT NOT NULL,
    is_enabled BOOLEAN DEFAULT 1,
    severity TEXT DEFAULT 'info',
    notify_desktop BOOLEAN DEFAULT 1,
    created_at TEXT DEFAULT (datetime('now'))
);

-- App settings (key-value store)
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Indexes
CREATE INDEX idx_devices_mac ON devices(mac_address);
CREATE INDEX idx_device_ips_current ON device_ips(device_id) WHERE is_current = 1;
CREATE INDEX idx_device_ips_address ON device_ips(ip_address) WHERE is_current = 1;
CREATE INDEX idx_device_ports_device ON device_ports(device_id, scan_id);
CREATE INDEX idx_latency_device_time ON latency_history(device_id, measured_at DESC);
CREATE INDEX idx_alerts_unread ON alerts(is_read) WHERE is_read = 0;
CREATE INDEX idx_scans_started ON scans(started_at DESC);

-- Seed default alert rules
INSERT INTO alert_rules (id, rule_type, is_enabled, severity, notify_desktop) VALUES
    ('rule_new_device', 'new_device', 1, 'info', 1),
    ('rule_device_departed', 'device_departed', 1, 'info', 0),
    ('rule_port_changed', 'port_changed', 1, 'warning', 1),
    ('rule_untrusted_device', 'untrusted_device', 1, 'warning', 1);

-- Seed default settings
INSERT INTO settings (key, value) VALUES
    ('scan_interval_secs', '60'),
    ('port_range', 'top100'),
    ('theme', 'dark'),
    ('graph_repulsion', '300'),
    ('graph_link_distance', '100'),
    ('graph_gravity', '0.1');
