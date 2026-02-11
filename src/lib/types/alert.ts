export type AlertEventType = 'new_device' | 'device_departed' | 'port_changed' | 'unknown_device';

export type AlertRuleType = 'new_device' | 'device_departed' | 'port_changed' | 'untrusted_device';

export type Severity = 'info' | 'warning' | 'critical';

export interface Alert {
	id: string;
	alertType: AlertEventType;
	deviceId: string | null;
	message: string;
	severity: Severity;
	isRead: boolean;
	createdAt: string;
}

export interface AlertRule {
	id: string;
	ruleType: AlertRuleType;
	isEnabled: boolean;
	severity: Severity;
	notifyDesktop: boolean;
}

export interface AlertRuleUpdate {
	isEnabled?: boolean;
	severity?: Severity;
	notifyDesktop?: boolean;
}
