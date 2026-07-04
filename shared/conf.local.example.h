// conf.local.example.h
//
// Copy this file to shared/conf.local.h and fill in real values.
// shared/conf.local.h is gitignored; this template is committed.
//
// shared/conf.h auto-includes shared/conf.local.h via __has_include, so any
// macros defined here override the safe placeholders in conf.h.

#pragma once

// Flip to 1 to compile in the WiFi + remote-write code path. You can also
// set this from core/platformio.ini via build_flags = -DENABLE_PROMETHEUS=1.
#define ENABLE_PROMETHEUS 1

// Per-device identity. Used as the Prometheus `instance` label.
#define PROM_DEVICE_ID "co2nsole-livingroom"

// WiFi (2.4 GHz only — ESP32 does not support 5 GHz).
#define PROM_WIFI_SSID "your-ssid"
#define PROM_WIFI_PASS "your-wifi-password"

// Grafana Cloud remote-write endpoint.
//
// Find these values at:
//   grafana.com -> My Account -> select your stack -> "Details" under
//   "Prometheus" -> "Remote Write Endpoint".
//
// The URL Grafana shows is e.g.
//   https://prometheus-prod-37-prod-eu-west-2.grafana.net/api/prom/push
// Split it into HOST (no scheme, no path) and PATH.
//
// PROM_GC_USER is the numeric "Username / Instance ID".
// PROM_GC_PASS is the API key created under "API Keys" with role
//   `MetricsPublisher` and scope `metrics:write`.
#define PROM_GC_URL  "prometheus-prod-XX-prod-eu-west-X.grafana.net"
#define PROM_GC_PATH "/api/prom/push"
#define PROM_GC_PORT 443
#define PROM_GC_USER "123456"
#define PROM_GC_PASS "glc_eyJrIjo...replace-with-real-key"
