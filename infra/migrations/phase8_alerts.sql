-- Phase 8: Alerts & Notifications Schema

CREATE TABLE IF NOT EXISTS alert_rules (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    url_pattern TEXT NOT NULL,
    categories JSONB NOT NULL, -- Array of SemanticCategory strings
    channels JSONB NOT NULL,   -- Array of NotificationChannel objects
    min_confidence FLOAT DEFAULT 0.5,
    active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    last_triggered_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS notification_history (
    id UUID PRIMARY KEY,
    rule_id UUID REFERENCES alert_rules(id),
    url TEXT NOT NULL,
    channel_type TEXT NOT NULL,
    status TEXT NOT NULL,
    delivered_at TIMESTAMPTZ DEFAULT NOW(),
    payload JSONB
);

CREATE INDEX IF NOT EXISTS idx_alert_rules_active ON alert_rules(active);
