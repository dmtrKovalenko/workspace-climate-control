// Prometheus remote-write reporter for the climate sensor.
//
// Climate-only metrics (no device-health series). Wraps grafana's
// PrometheusArduino + PromLokiTransport so the rest of the firmware sees a
// tiny surface: begin() once, capture() per sensor cycle, maybeSend() to flush
// on the configured cadence.
//
// Configuration is pulled from shared/conf.h (which auto-includes
// shared/conf.local.h for secrets). Compile-time gate: ENABLE_PROMETHEUS.

#pragma once

#include "../../shared/conf.h"

#if ENABLE_PROMETHEUS

#include "data.h"
#include <PromLokiTransport.h>
#include <PrometheusArduino.h>

class PrometheusReporter {
public:
  PrometheusReporter();

  // Bounded init: WiFi (<= PROM_WIFI_RECONNECT_TIMEOUT_MS) + SNTP
  // (<= PROM_NTP_SYNC_TIMEOUT_MS) + TLS client wiring. Never blocks forever —
  // if the AP or the internet is unavailable it returns false and BLE
  // proceeds; maybeInit() retries later. Call once from setup() *before*
  // BLEDevice::init to keep heap fragmentation in check.
  bool begin();

  // Lazy retry for a failed begin(): re-attempts every PROM_INIT_RETRY_MS.
  // No-op once initialized. Call from loop(); a retry blocks the loop for at
  // most WiFi+NTP timeout (~45 s) which the task WDT budget covers.
  void maybeInit();

  // Append one sample per metric to the in-RAM batch. Cheap; safe to call
  // from the regular 2 s sensor loop — internally throttled to one effective
  // sample per PROM_CAPTURE_EVERY_MS (set in shared/conf.h). No-op if begin()
  // failed.
  void capture(const ClimateData &data);

  // Flush the batch over remote-write if PROM_SEND_EVERY_MS has elapsed since
  // the last send. Self-healing: performs a *bounded* WiFi reconnect when the
  // AP dropped us, resets stale batches after PROM_FAILS_BEFORE_BATCH_RESET
  // consecutive failures, and calls ESP.restart() after
  // PROM_FAILS_BEFORE_REBOOT. Returns true on a successful POST.
  bool maybeSend();

  const char *lastError() const { return errmsg_ ? errmsg_ : ""; }

private:
  static constexpr uint8_t kSeriesCount = 11;

  // Bounded replacement for the upstream transport's infinite reconnect
  // loop. Returns true when WiFi is usable.
  bool ensureWifi();
  // Bounded SNTP sync (upstream's is an infinite loop). True once the RTC
  // holds plausible wall-clock time.
  bool syncClock();
  // One-time TLS client + series registration; idempotent.
  bool wireClient();
  void resetAllBatches();
  void onSendFailure(const char *what);

  bool initialized_;
  bool wired_;
  bool boot_banner_printed_;
  uint32_t last_init_attempt_ms_;
  uint32_t last_capture_ms_;
  uint32_t last_send_ms_;
  uint32_t fail_streak_;
  uint32_t send_failures_total_;
  const char *errmsg_;

  PromLokiTransport transport_;
  PromClient client_;
  WriteRequest req_;

  TimeSeries ts_co2_;
  TimeSeries ts_eco2_;
  TimeSeries ts_etvoc_;
  TimeSeries ts_temperature_;
  TimeSeries ts_humidity_;
  TimeSeries ts_pressure_;
  TimeSeries ts_light_;
  // Device-health series (see shared/conf.h for why these exist).
  TimeSeries ts_heap_;
  TimeSeries ts_rssi_;
  TimeSeries ts_uptime_;
  TimeSeries ts_send_failures_;
};

#endif // ENABLE_PROMETHEUS
