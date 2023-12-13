#include "Adafruit_TinyUSB.h"
#include "Adafruit_VL53L0X.h"
#include "battery.h"
#include <bluefruit.h>

#define CHARGIN_CURRENT_CONTROL_PIN 13

SoftwareTimer mainTimer;
Adafruit_VL53L0X lox = Adafruit_VL53L0X();

BLEService service("19B10000-E8F2-537E-4F6C-D104768A1214");
BLECharacteristic characteristic("19B10000-E8F2-537E-4F6C-D104768A1214");

BLEDis bledis; // DIS (Device Information Service) helper class instance
Battery battery;

void prepareBluetooth() {
  Bluefruit.setName("CO2CICKA WINDOWS");

  Bluefruit.Advertising.addFlags(BLE_GAP_ADV_FLAGS_LE_ONLY_GENERAL_DISC_MODE);
  Bluefruit.Advertising.addTxPower();

  // There is enough room for the dev name in the advertising packet
  Bluefruit.Advertising.addName();

  bledis.setManufacturer("Adafruit Industries");
  bledis.setModel("Bluefruit Feather52");
  bledis.begin();
  battery.begin();

  Bluefruit.Advertising.addService(service);
  service.begin();

  characteristic.setProperties(CHR_PROPS_READ | CHR_PROPS_NOTIFY);
  characteristic.setPermission(SECMODE_OPEN, SECMODE_NO_ACCESS);
  characteristic.setFixedLen(1);
  characteristic.begin();

  /* Start Advertising
   * - Enable auto advertising if disconnected
   * - Interval:  fast mode = 20 ms, slow mode = 152.5 ms
   * - Timeout for fast mode is 30 seconds
   * - Start(timeout) with timeout = 0 will advertise forever (until connected)
   *
   * For recommended advertising interval
   * https://developer.apple.com/library/content/qa/qa1931/_index.html
   */
  Bluefruit.Advertising.restartOnDisconnect(false);
  Bluefruit.Advertising.setInterval(500, 500); // in unit of 0.625 ms
  Bluefruit.autoConnLed(false);
}

void measurementCallback(TimerHandle_t xTimerID) {
  xTimerReset(xTimerID, 0);

  VL53L0X_RangingMeasurementData_t measure;

  Serial.println("Trying to update battery service");
  battery.updateBatteryState();
  Serial.println("Updated battery service");

  Serial.println("Reading a measurement... ");
  lox.rangingTest(&measure,
                  false); // pass in 'trz mue' to get debug data printout!
  Serial.print("Measurement done:");

  uint8_t notify_value = 0;

  if (measure.RangeStatus != 4) { // phase failures have incorrect data
    Serial.print("Distance (mm): ");
    Serial.println(measure.RangeMilliMeter);

    bool isClosed = measure.RangeMilliMeter < 100;
    notify_value = isClosed;

    if (isClosed) {
      digitalWrite(LED_RED, LOW);
    } else {
      digitalWrite(LED_RED, HIGH);
    }
  } else {
    notify_value = false;
    digitalWrite(LED_RED, HIGH);
    Serial.println(" out of range ");
  }

  characteristic.write8(notify_value);

  digitalWrite(LED_BLUE, LOW);
  Bluefruit.Advertising.start();
  delay(5000);
  digitalWrite(LED_BLUE, HIGH);

  if (Bluefruit.connected()) {
    Serial.println("BLE connected, writing and waiting for disconnect");

    while (Bluefruit.connected()) {
      delay(100);
      digitalWrite(LED_BLUE, !digitalRead(LED_BLUE));
    }

    digitalWrite(LED_BLUE, HIGH);
  } else {
    Bluefruit.Advertising.stop();
  }

  xTimerStart(xTimerID, 0);
  sd_app_evt_wait();
}

void setup() {
  Serial.begin(115200);
  // while (!Serial)
  //   ;

  pinMode(LED_RED, OUTPUT);
  pinMode(LED_BLUE, OUTPUT);

  pinMode(CHARGIN_CURRENT_CONTROL_PIN, OUTPUT);
  digitalWrite(CHARGIN_CURRENT_CONTROL_PIN, HIGH);

  if (!lox.begin()) {
    Serial.println(F("Failed to boot VL53L0X"));
    while (1)
      ;
  }

  digitalWrite(LED_RED, LOW);
  // power
  Bluefruit.begin();
  Bluefruit.setTxPower(-8); // Check bluefruit.h for supported values

  prepareBluetooth();
  delay(100);
  digitalWrite(LED_RED, HIGH);
  Serial.println("Starting loop");
  sd_power_mode_set(NRF_POWER_MODE_LOWPWR);

  mainTimer.begin(30000, measurementCallback);
  mainTimer.start();
  suspendLoop();

  sd_app_evt_wait();
}

void loop() {}
