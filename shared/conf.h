// Core sensor BLE
#define BLE_MAIN_SERVICE_LOCAL_NAME "co2nsole"
#define BLE_MAIN_SENSOR_SERVICE "0000FFE0-0000-1000-8000-00805F9B34FB"
#define BLE_MAIN_SENSOR_STREAM_CHAR "0000FFE0-0000-1000-8000-00805F9B34FB"
#define BLE_MAIN_SENSOR_CO2_CALIBRATION_CHAR                                   \
  "beb5483e-36e1-4688-b7f5-ea07361b26a8"
#define BLE_MAIN_SENSOR_TEMP_CALIBRATION_CHAR                                  \
  "d753f24d-3aa0-4678-b039-a52d3b2e3946"

#define BLE_MAIN_SENSOR_CALIBRATE_CO2 "CalibrateMhZ19"
#define BLE_MAIN_SENSOR_CALIBRATE_TEMPERATURE "CalibrateTemperature"
#define BLE_MAIN_SENSOR_CALIBRATE_HUMIDITY "CalibrateHumidity"

// Window sensor BLE
#define BLE_WINDOW_SERVICE_LOCAL_NAME "co2nsole window"
#define BLE_WINDOW_SENSOR_SERVICE "19B10000-E8F2-537E-4F6C-D104768A1214"
#define BLE_WINDOW_SENSOR_READ_CHAR "a3b27688-3b9e-4214-970c-3db5f14c5d2b"

// Calibrator defaults
#define CALIBRATION_CO2_DEFAULT 400
#define CALIBRATION_TEMPERATURE_ADJUST -8

// ---------------------------------------------------------------------------
// Prometheus remote-write (climate metrics fan-out)
// ---------------------------------------------------------------------------
//
// Enable by creating shared/conf.local.h (gitignored) from the example file
// and filling in real WiFi + Grafana Cloud secrets. ENABLE_PROMETHEUS is
// auto-derived: if conf.local.h does not define PROM_GC_URL, we fall back to
// 0 and the entire WiFi/TLS code path is excluded from the build. PIO's
// `lib_ldf_mode = chain+` then leaves the grafana/* lib_deps downloaded but
// uncompiled, so the BLE-only build stays small.

// Pull in local secrets first; everything below derives from what they set.
#if defined(__has_include)
#  if __has_include("conf.local.h")
#    include "conf.local.h"
#  endif
#endif

// Auto-derive the toggle from secret presence. Even if -DENABLE_PROMETHEUS=1
// is forced via build_flags, a missing PROM_GC_URL pushes it back off — we
// won't ship a binary that fails at runtime trying to connect to
// "<UNDEFINED>".
#ifdef PROM_GC_URL
#  ifndef ENABLE_PROMETHEUS
#    define ENABLE_PROMETHEUS 1
#  endif
#else
#  ifdef ENABLE_PROMETHEUS
#    undef ENABLE_PROMETHEUS
#  endif
#  define ENABLE_PROMETHEUS 0
#endif

// Per-device identity. Becomes the Prometheus `instance` label so multiple
// physical units can report into the same backend without colliding. Override
// per-unit in conf.local.h.
#ifndef PROM_DEVICE_ID
#  define PROM_DEVICE_ID "co2nsole-001"
#endif

// Metric names. Single source of truth (kept here so cli/ can reference them
// via bindgen if it ever needs to). Naming follows
// https://prometheus.io/docs/practices/naming/ (app prefix + unit suffix).
#define PROM_METRIC_CO2         "co2nsole_co2_ppm"
#define PROM_METRIC_ECO2        "co2nsole_eco2_ppm"
#define PROM_METRIC_ETVOC       "co2nsole_etvoc_ppb"
#define PROM_METRIC_TEMPERATURE "co2nsole_temperature_celsius"
#define PROM_METRIC_HUMIDITY    "co2nsole_humidity_percent"
#define PROM_METRIC_PRESSURE    "co2nsole_pressure_hpa"
#define PROM_METRIC_LIGHT       "co2nsole_light_lux"

// Device-health metrics. These exist to make reliability incidents
// diagnosable from Grafana itself:
//   - uptime sawtooth (drops to 0)  -> device rebooted (watchdog/escalation)
//   - rssi dips                      -> weak WiFi, expect gaps
//   - heap decline                   -> leak/fragmentation, expect a reboot
//   - send_failures jumps            -> network outage window (counter is
//                                       only visible after recovery, but the
//                                       delta tells you how bad it was)
#define PROM_METRIC_HEAP          "co2nsole_heap_free_bytes"
#define PROM_METRIC_RSSI          "co2nsole_wifi_rssi_dbm"
#define PROM_METRIC_UPTIME        "co2nsole_uptime_seconds"
#define PROM_METRIC_SEND_FAILURES "co2nsole_send_failures_total"

// Sizing / cadence.
//   - PROM_CAPTURE_EVERY_MS: throttle for prometheus.capture() — sensors are
//                            still read every 2s for BLE, but we only record
//                            one Prometheus sample per metric per this many
//                            milliseconds.
//   - PROM_BATCH_SAMPLES:    in-RAM samples per series before addSample drops.
//                            Sized for one extra slot beyond what a normal
//                            send drains, giving retry headroom.
//   - PROM_BUFFER_BYTES:     proto+snappy buffer size (allocated 2x on stack
//                            during send). Bump if upstream debug logs report
//                            "Required buffer size for compression: N" > this
//                            or "Error from proto encode: stream full".
//   - PROM_SEND_EVERY_MS:    cadence of remote-write POSTs.
#define PROM_CAPTURE_EVERY_MS 30000
#define PROM_BATCH_SAMPLES    3
#define PROM_BUFFER_BYTES     2048
#define PROM_SEND_EVERY_MS    60000

// Reliability / self-healing.
//   - PROM_WIFI_RECONNECT_TIMEOUT_MS: max time a WiFi reconnect attempt may
//     block the loop (upstream's own helper spins forever — we don't).
//   - PROM_FAILS_BEFORE_BATCH_RESET: consecutive failed sends before we drop
//     buffered samples. Prevents pushing stale/out-of-order data after a
//     long outage (Grafana Cloud rejects samples outside its OOO window).
//   - PROM_FAILS_BEFORE_REBOOT: consecutive failed sends before ESP.restart().
//     With 60 s sends, 10 fails ~= 10 minutes of outage -> reboot gives a
//     fresh WiFi/TLS/lwip stack, which clears wedge states that reconnects
//     can't (a known ESP32 BLE+WiFi coex failure mode).
//   - PROM_WDT_TIMEOUT_S: task watchdog. If the loop task hangs (TLS stall,
//     upstream infinite reconnect loop, lwip deadlock) the chip reboots
//     instead of staying silent forever.
#define PROM_WIFI_RECONNECT_TIMEOUT_MS 15000
#define PROM_FAILS_BEFORE_BATCH_RESET  3
#define PROM_FAILS_BEFORE_REBOOT       10
#define PROM_WDT_TIMEOUT_S             120

// Bounded init (upstream transport.begin() loops *forever* on WiFi connect
// and NTP sync — a device that reboots while the AP or the internet is down
// would otherwise never finish setup(), killing BLE too).
//   - PROM_NTP_SERVER:          SNTP pool used to set the clock (TLS needs it).
//   - PROM_NTP_SYNC_TIMEOUT_MS: max wait for the first time sync per attempt.
//   - PROM_INIT_RETRY_MS:       if init failed (no WiFi/NTP at boot), retry
//                               from loop() this often. BLE runs regardless.
#define PROM_NTP_SERVER          "pool.ntp.org"
#define PROM_NTP_SYNC_TIMEOUT_MS 30000
#define PROM_INIT_RETRY_MS       300000

// Dummy fallbacks. Only used to keep the firmware and the cli/ bindgen pass
// compiling when conf.local.h is absent. The "<UNDEFINED>" sentinel makes it
// obvious in serial logs (and in raw_bindings.rs, if it weren't blocklisted)
// that secrets weren't supplied.
#ifndef PROM_WIFI_SSID
#  define PROM_WIFI_SSID "<UNDEFINED>"
#endif
#ifndef PROM_WIFI_PASS
#  define PROM_WIFI_PASS "<UNDEFINED>"
#endif
#ifndef PROM_GC_URL
#  define PROM_GC_URL "<UNDEFINED>"
#endif
#ifndef PROM_GC_PATH
#  define PROM_GC_PATH "/api/prom/push"
#endif
#ifndef PROM_GC_PORT
#  define PROM_GC_PORT 443
#endif
#ifndef PROM_GC_USER
#  define PROM_GC_USER "<UNDEFINED>"
#endif
#ifndef PROM_GC_PASS
#  define PROM_GC_PASS "<UNDEFINED>"
#endif