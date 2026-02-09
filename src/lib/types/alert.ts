export type AlertType = 'newDevice' | 'deviceDeparted' | 'portChanged' | 'unknownDevice';

export type Severity = 'info' | 'warning' | 'critical';

export interface Alert {
	id: string;
	alertType: AlertType;
	deviceId: string | null;
	message: string;
	severity: Severity;
	isRead: boolean;
	createdAt: string;
}

export interface AlertRule {
	id: string;
	ruleType: AlertType;
	isEnabled: boolean;
	severity: Severity;
	notifyDesktop: boolean;
}

export interface AlertRuleUpdate {
	isEnabled?: boolean;
	severity?: Severity;
	notifyDesktop?: boolean;
}
