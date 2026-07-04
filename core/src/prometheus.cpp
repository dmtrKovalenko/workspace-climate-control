#include "prometheus.h"

#if ENABLE_PROMETHEUS

#include "certificates.h"
#include <Arduino.h>
#include <ArduinoBearSSL.h>
#include <WiFi.h>
#include <cstring>
#include <esp_system.h>
#include <time.h>

#include "esp_sntp.h"

// Compile-time concatenation of the common label set. Adjacent C string
// literals are merged, so this becomes a single literal at compile time.
#define PROM_LABELS "{job=\"co2nsole\",instance=\"" PROM_DEVICE_ID "\"}"

// BearSSL asks for wall-clock time during certificate validation. Same
// implementation as upstream ESP32Client's getTime().
static unsigned long bearSslGetTime() {
  struct timeval tv_now;
  gettimeofday(&tv_now, NULL);
  return (unsigned long)tv_now.tv_sec;
}

// The RTC starts at epoch 0 on cold boot; anything past 2020-09 means SNTP
// (or a previous sync, surviving ESP.restart) has set it.
static bool clockIsSet() { return time(nullptr) > 1600000000; }

static const char *resetReasonStr(esp_reset_reason_t r) {
  switch (r) {
  case ESP_RST_POWERON:
    return "poweron";
  case ESP_RST_SW:
    return "software (ESP.restart)";
  case ESP_RST_PANIC:
    return "panic/exception";
  case ESP_RST_INT_WDT:
    return "interrupt watchdog";
  case ESP_RST_TASK_WDT:
    return "task watchdog";
  case ESP_RST_WDT:
    return "other watchdog";
  case ESP_RST_BROWNOUT:
    return "brownout";
  default:
    return "other";
  }
}

PrometheusReporter::PrometheusReporter()
    : initialized_(false), wired_(false), boot_banner_printed_(false),
      last_init_attempt_ms_(0), last_capture_ms_(0), last_send_ms_(0),
      fail_streak_(0), send_failures_total_(0), errmsg_(nullptr),
      client_(transport_), req_(kSeriesCount, PROM_BUFFER_BYTES),
      ts_co2_(PROM_BATCH_SAMPLES, PROM_METRIC_CO2, PROM_LABELS),
      ts_eco2_(PROM_BATCH_SAMPLES, PROM_METRIC_ECO2, PROM_LABELS),
      ts_etvoc_(PROM_BATCH_SAMPLES, PROM_METRIC_ETVOC, PROM_LABELS),
      ts_temperature_(PROM_BATCH_SAMPLES, PROM_METRIC_TEMPERATURE, PROM_LABELS),
      ts_humidity_(PROM_BATCH_SAMPLES, PROM_METRIC_HUMIDITY, PROM_LABELS),
      ts_pressure_(PROM_BATCH_SAMPLES, PROM_METRIC_PRESSURE, PROM_LABELS),
      ts_light_(PROM_BATCH_SAMPLES, PROM_METRIC_LIGHT, PROM_LABELS),
      ts_heap_(PROM_BATCH_SAMPLES, PROM_METRIC_HEAP, PROM_LABELS),
      ts_rssi_(PROM_BATCH_SAMPLES, PROM_METRIC_RSSI, PROM_LABELS),
      ts_uptime_(PROM_BATCH_SAMPLES, PROM_METRIC_UPTIME, PROM_LABELS),
      ts_send_failures_(PROM_BATCH_SAMPLES, PROM_METRIC_SEND_FAILURES,
                        PROM_LABELS) {}

bool PrometheusReporter::wireClient() {
  if (wired_) {
    return true;
  }

  // Transport object only supplies TLS client factories + timestamps for us;
  // we deliberately never call transport_.begin() — its WiFi connect and
  // SNTP loops are UNBOUNDED upstream and would wedge setup() (and BLE with
  // it) whenever the AP or the internet is down at boot.
  transport_.setUseTls(true);
  transport_.setCerts(grafanaCert, strlen(grafanaCert));
  transport_.setWifiSsid(PROM_WIFI_SSID);
  transport_.setWifiPass(PROM_WIFI_PASS);
  transport_.setDebug(Serial);

  // Remote-write client (Grafana Cloud auth is HTTP basic with numeric user
  // + API key). setPath takes a non-const char* — match the upstream
  // example's cast.
  client_.setUrl(PROM_GC_URL);
  client_.setPath((char *)PROM_GC_PATH);
  client_.setPort(PROM_GC_PORT);
  client_.setUser(PROM_GC_USER);
  client_.setPass(PROM_GC_PASS);
  client_.setDebug(Serial);

  if (!client_.begin()) {
    errmsg_ = client_.errmsg;
    Serial.print("Prometheus client.begin failed: ");
    Serial.println(errmsg_ ? errmsg_ : "(no errmsg)");
    return false;
  }

  // Register every series with the WriteRequest. addTimeSeries returns false
  // if we exceed the kSeriesCount cap declared at construction time — that
  // would be a coding error, not a runtime fault. Must run exactly once.
  if (!req_.addTimeSeries(ts_co2_) || !req_.addTimeSeries(ts_eco2_) ||
      !req_.addTimeSeries(ts_etvoc_) ||
      !req_.addTimeSeries(ts_temperature_) ||
      !req_.addTimeSeries(ts_humidity_) ||
      !req_.addTimeSeries(ts_pressure_) || !req_.addTimeSeries(ts_light_) ||
      !req_.addTimeSeries(ts_heap_) || !req_.addTimeSeries(ts_rssi_) ||
      !req_.addTimeSeries(ts_uptime_) ||
      !req_.addTimeSeries(ts_send_failures_)) {
    errmsg_ = req_.errmsg;
    Serial.print("Prometheus addTimeSeries failed: ");
    Serial.println(errmsg_ ? errmsg_ : "(no errmsg)");
    return false;
  }
  req_.setDebug(Serial);

  ArduinoBearSSL.onGetTime(bearSslGetTime);

  wired_ = true;
  return true;
}

bool PrometheusReporter::syncClock() {
  if (clockIsSet()) {
    return true;
  }

  if (!sntp_enabled()) {
    sntp_setoperatingmode(SNTP_OPMODE_POLL);
    sntp_setservername(0, (char *)PROM_NTP_SERVER);
    sntp_init();
  }

  Serial.print("Syncing clock from ");
  Serial.println(PROM_NTP_SERVER);
  const uint32_t deadline = millis() + PROM_NTP_SYNC_TIMEOUT_MS;
  while (!clockIsSet() && static_cast<int32_t>(deadline - millis()) > 0) {
    delay(250);
  }

  if (clockIsSet()) {
    Serial.println("Clock synced");
    return true;
  }
  Serial.println("NTP sync timed out");
  return false;
}

bool PrometheusReporter::begin() {
  if (initialized_) {
    return true;
  }
  last_init_attempt_ms_ = millis();

  if (!boot_banner_printed_) {
    boot_banner_printed_ = true;
    // Reset-reason breadcrumb: after a gap, this line in the serial log (or
    // the uptime metric dropping to 0 in Grafana) tells you whether the
    // firmware self-healed via watchdog/escalation reboot.
    Serial.print("Boot reset reason: ");
    Serial.println(resetReasonStr(esp_reset_reason()));
  }

  if (!wireClient()) {
    // Hard failure (misconfiguration) — retrying won't help, but it's cheap
    // and keeps the code path single.
    return false;
  }

  if (!ensureWifi()) {
    errmsg_ = "wifi unavailable";
    Serial.println("Prometheus init deferred: no WiFi (BLE unaffected; will "
                   "retry)");
    return false;
  }

  // TLS certificate validation needs wall-clock time.
  if (!syncClock()) {
    errmsg_ = "ntp sync timeout";
    Serial.println("Prometheus init deferred: no NTP (BLE unaffected; will "
                   "retry)");
    return false;
  }

  initialized_ = true;
  last_send_ms_ = millis();
  // Force the first capture to fire on the next loop tick by leaving
  // last_capture_ms_ at 0 — this seeds the dashboard within seconds of boot
  // rather than waiting a full PROM_CAPTURE_EVERY_MS.
  Serial.println("Prometheus reporter ready");
  return true;
}

void PrometheusReporter::maybeInit() {
  if (initialized_) {
    return;
  }
  if (last_init_attempt_ms_ != 0 &&
      millis() - last_init_attempt_ms_ < PROM_INIT_RETRY_MS) {
    return;
  }
  Serial.println("Prometheus: retrying deferred init");
  begin();
}

void PrometheusReporter::capture(const ClimateData &data) {
  if (!initialized_) {
    return;
  }

  // Throttle: the sensor loop calls us every 2 s for BLE freshness, but we
  // only want one Prometheus sample per metric per PROM_CAPTURE_EVERY_MS.
  const uint32_t now = millis();
  if (last_capture_ms_ != 0 && now - last_capture_ms_ < PROM_CAPTURE_EVERY_MS) {
    return;
  }
  last_capture_ms_ = now;

  const int64_t ts = transport_.getTimeMillis();

  // addSample returns false when the per-series batch is full. We log but
  // don't bail — the next maybeSend() will drain it.
  auto add = [&](TimeSeries &series, double value) {
    if (!series.addSample(ts, value)) {
      Serial.print("Prometheus addSample dropped (");
      Serial.print(series.errmsg ? series.errmsg : "?");
      Serial.println(")");
    }
  };

  add(ts_co2_, static_cast<double>(data.co2));
  add(ts_eco2_, static_cast<double>(data.eco2));
  add(ts_etvoc_, static_cast<double>(data.etvoc));
  add(ts_temperature_, static_cast<double>(data.temperature));
  add(ts_humidity_, static_cast<double>(data.humidity));
  add(ts_pressure_, static_cast<double>(data.pressure));
  add(ts_light_, static_cast<double>(data.light));

  // Device health.
  add(ts_heap_, static_cast<double>(ESP.getFreeHeap()));
  add(ts_rssi_, static_cast<double>(WiFi.RSSI()));
  add(ts_uptime_, static_cast<double>(millis()) / 1000.0);
  add(ts_send_failures_, static_cast<double>(send_failures_total_));
}

void PrometheusReporter::resetAllBatches() {
  ts_co2_.resetSamples();
  ts_eco2_.resetSamples();
  ts_etvoc_.resetSamples();
  ts_temperature_.resetSamples();
  ts_humidity_.resetSamples();
  ts_pressure_.resetSamples();
  ts_light_.resetSamples();
  ts_heap_.resetSamples();
  ts_rssi_.resetSamples();
  ts_uptime_.resetSamples();
  ts_send_failures_.resetSamples();
}

bool PrometheusReporter::ensureWifi() {
  if (WiFi.status() == WL_CONNECTED) {
    return true;
  }

  // Upstream's checkAndReconnectConnection() spins forever while the AP is
  // down, which would wedge BLE alongside Prometheus. Reconnect with a hard
  // deadline instead; if the AP is still gone we just skip this send cycle
  // and let the failure-streak escalation handle persistent outages.
  Serial.println("WiFi down; attempting bounded reconnect");
  WiFi.disconnect();
  delay(10);
  WiFi.mode(WIFI_STA);
  WiFi.begin(PROM_WIFI_SSID, PROM_WIFI_PASS);

  const uint32_t deadline = millis() + PROM_WIFI_RECONNECT_TIMEOUT_MS;
  while (WiFi.status() != WL_CONNECTED &&
         static_cast<int32_t>(deadline - millis()) > 0) {
    delay(250);
  }

  if (WiFi.status() == WL_CONNECTED) {
    Serial.print("WiFi reconnected, IP: ");
    Serial.println(WiFi.localIP());
    return true;
  }

  Serial.println("WiFi reconnect timed out");
  return false;
}

void PrometheusReporter::onSendFailure(const char *what) {
  fail_streak_++;
  send_failures_total_++;

  Serial.print("Prometheus send failure #");
  Serial.print(fail_streak_);
  Serial.print(": ");
  Serial.println(what ? what : "(unknown)");

  // Long outage: buffered samples are stale by now and Grafana Cloud will
  // reject them as out-of-order once we recover — drop them so the first
  // post-recovery send is clean.
  if (fail_streak_ == PROM_FAILS_BEFORE_BATCH_RESET) {
    Serial.println("Prometheus: dropping stale batches after repeated "
                   "failures");
    resetAllBatches();
  }

  // Very long outage: reconnects aren't fixing it. A reboot rebuilds
  // WiFi/TLS/lwip from scratch, which recovers wedge states nothing else
  // can. BLE clients reconnect automatically via the CLI's retry loop.
  if (fail_streak_ >= PROM_FAILS_BEFORE_REBOOT) {
    Serial.println("Prometheus: unrecoverable send failures, restarting "
                   "device");
    Serial.flush();
    delay(100);
    ESP.restart();
  }
}

bool PrometheusReporter::maybeSend() {
  if (!initialized_) {
    return false;
  }
  if (millis() - last_send_ms_ < PROM_SEND_EVERY_MS) {
    return false;
  }
  last_send_ms_ = millis();

  if (!ensureWifi()) {
    onSendFailure("wifi unavailable");
    return false;
  }

  PromClient::SendResult res = client_.send(req_);

  if (res == PromClient::SendResult::SUCCESS) {
    fail_streak_ = 0;
    resetAllBatches();
    Serial.println("Prometheus send OK");
    return true;
  }

  // FAILED_DONT_RETRY means the server actively rejected the payload (4xx:
  // bad auth, out-of-order samples, ...). Retrying the same batch would fail
  // identically, so drop it; fresh samples may succeed.
  if (res == PromClient::SendResult::FAILED_DONT_RETRY) {
    resetAllBatches();
  }

  errmsg_ = client_.errmsg;
  onSendFailure(errmsg_);
  return false;
}

#endif // ENABLE_PROMETHEUS
