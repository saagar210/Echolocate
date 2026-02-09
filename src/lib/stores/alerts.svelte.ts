import { writable, derived } from 'svelte/store';
import type { Alert, AlertRule } from '$lib/types/alert';

/** All alerts */
export const alerts = writable<Alert[]>([]);

/** Alert rules configuration */
export const alertRules = writable<AlertRule[]>([]);

/** Unread alert count */
export const unreadCount = derived(alerts, ($alerts) =>
	$alerts.filter((a) => !a.isRead).length
);

/** Whether any critical unread alerts exist */
export const hasCritical = derived(alerts, ($alerts) =>
	$alerts.some((a) => !a.isRead && a.severity === 'critical')
);

/** Replace all alerts (initial load) */
export function setAlerts(list: Alert[]): void {
	alerts.set(list);
}

/** Add a new alert (prepend — newest first) */
export function addAlert(alert: Alert): void {
	alerts.update((list) => [alert, ...list]);
}

/** Mark a single alert as read */
export function markRead(alertId: string): void {
	alerts.update((list) =>
		list.map((a) => (a.id === alertId ? { ...a, isRead: true } : a))
	);
}

/** Mark all alerts as read */
export function markAllRead(): void {
	alerts.update((list) => list.map((a) => ({ ...a, isRead: true })));
}

/** Replace alert rules (initial load) */
export function setAlertRules(rules: AlertRule[]): void {
	alertRules.set(rules);
}
