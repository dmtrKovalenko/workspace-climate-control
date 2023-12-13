#include <Arduino.h>
#include <bluefruit.h>

const double vRef = 2.4;
const unsigned int numReadings =
    1024; // 10-bit ADC readings 0-1023, so the factor is 1024

const float MIN_VOLTAGE = 3.6;  // Minimum battery voltage
const float MAX_VOLTAGE = 4.1;  // Maximum battery voltage
const int MIN_PERCENTAGE = 0;   // Minimum percentage value
const int MAX_PERCENTAGE = 100; // Maximum percentage value

#define BAT_CHARGE_STATE 23 // LOW for charging, HIGH not charging

class Battery {
public:
  BLEService batteryBleService;
  BLECharacteristic chargeCharacteristic;
  BLECharacteristic isChargingCharacteristic;
  float lastChargePercentage;

  Battery();
  void begin();
  void updateBatteryState();
  bool isCharging();
};

Battery::Battery() {
  analogReference(AR_INTERNAL_2_4);
  pinMode(VBAT_ENABLE, OUTPUT);
  pinMode(BAT_CHARGE_STATE, INPUT);

  batteryBleService = BLEService("1a1a1963-7e2b-45ed-a1f7-82d01423e841");
  chargeCharacteristic =
      BLECharacteristic("aaa8d3ef-4c86-47a2-9428-6bdcf6041e3e");
  isChargingCharacteristic =
      BLECharacteristic("d6bc7685-31c6-4fd7-843f-9e74237ca2d2");

  digitalWrite(VBAT_ENABLE, LOW);
}

void startCharacteristic(BLECharacteristic &characteristic) {
  characteristic.setProperties(CHR_PROPS_READ | CHR_PROPS_NOTIFY);
  characteristic.setPermission(SECMODE_OPEN, SECMODE_NO_ACCESS);
  characteristic.setFixedLen(1);
  characteristic.begin();
}

void Battery::begin() {
  Bluefruit.Advertising.addService(batteryBleService);
  batteryBleService.begin();

  startCharacteristic(chargeCharacteristic);
  startCharacteristic(isChargingCharacteristic);

  lastChargePercentage = 100.0;
  this->chargeCharacteristic.notify8(100.0);
}

// Function to map the battery voltage to a percentage
uint8_t mapVoltageToPercentage(float voltage) {
  if (voltage < MIN_VOLTAGE) {
    return -1;
  }

  // My battery is kind of jank and old so the values are experimental while the
  // real battery capacity and voltage remains unknown
  if (voltage > MAX_VOLTAGE) {
    return 100;
  }

  // Map the voltage to the percentage scale
  float percentage =
      ((voltage - MIN_VOLTAGE) / (MAX_VOLTAGE - MIN_VOLTAGE)) * 100;
  return round(percentage);
}

bool Battery::isCharging() { return digitalRead(BAT_CHARGE_STATE) == LOW; }

void Battery::updateBatteryState() {
  unsigned int adcCount = analogRead(PIN_VBAT);
  double adcVoltage = (adcCount * vRef) / numReadings;
  double vBat = adcVoltage * 1510.0 / 510.0; // Voltage divider from Vbat to ADC

  uint8_t percentage = mapVoltageToPercentage(vBat);
  bool isCharging = this->isCharging();
  Serial.printf("Charge %d", percentage);

  this->isChargingCharacteristic.notify8(isCharging);

  if (percentage < lastChargePercentage || isCharging) {
    this->chargeCharacteristic.notify8(percentage);
    Serial.println("Notified");
    lastChargePercentage = percentage;
  }
}