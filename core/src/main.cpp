#include "Adafruit_BMP280.h"
#include "Adafruit_Si7021.h"
#include "HardwareSerial.h"
#include "ble.cpp"
#include "ccs811.h"
#include "i2c_scanner.h"
#include <Adafruit_Sensor.h>
#include <Arduino.h>
#include <BH1750.h>
#include <FirebaseJson.h>
#include <Wire.h>

#define BAUDRATE 9600

CCS811 ccs811;
Adafruit_BMP280 bmp280; // I2C
Adafruit_Si7021 SI702x = Adafruit_Si7021();
BH1750 lightSensor(0x23);
MHZ19 mhZ19;

HardwareSerial mySerial(2);

BleProtocol bleProtocol;

void setup() {
  Serial.begin(BAUDRATE);

  Serial.println("DHT22 Temperature and Humidity Sensor");
  Serial.println("------------------------------------");

  Wire.begin(21, 22);
  mySerial.begin(BAUDRATE);

  if (!lightSensor.begin(BH1750::CONTINUOUS_HIGH_RES_MODE, 0x23)) {
    Serial.println("Error initializing BH1750");
  }

  ccs811.set_i2cdelay(50); // Needed for ESP8266 because it doesn't handle I2C
                           // clock stretch correctly
  if (!ccs811.begin()) {
    Serial.println("Failed to start CSS811! Please check your wiring.");
  }

  Serial.print("setup: hardware    version: ");
  Serial.println(ccs811.hardware_version(), HEX);
  Serial.print("setup: bootloader  version: ");
  Serial.println(ccs811.bootloader_version(), HEX);
  Serial.print("setup: application version: ");
  Serial.println(ccs811.application_version(), HEX);

  // Start measuring
  bool ok = ccs811.start(CCS811_MODE_1SEC);
  if (!ok)
    Serial.println("setup: CCS811 start FAILED");

  Serial.println("BMP280 test"); /* --- SETUP BMP on 0x76 ------ */
  if (!bmp280.begin(0x76)) {
    Serial.println("Could not find a valid BMP280 sensor, check wiring!");
  }

  Serial.println("Si7021 test!"); /* ---- SETUP SI702x ----- */
  if (!SI702x.begin()) {
    Serial.println("Did not find Si702x sensor!");
  }

  Serial.print(" Revision(");
  Serial.print(SI702x.getRevision());
  Serial.print(")");
  Serial.print(" Serial #");
  Serial.print(SI702x.sernum_a, HEX);
  Serial.println(SI702x.sernum_b, HEX);

  while (mhZ19.errorCode != RESULT_OK) {
    Serial.println("Estabilshing connection to MH-Z19");
    mhZ19.begin(mySerial);
  };

  delay(2000);
  mhZ19.calibrate();
  bleProtocol.setup(pSensors{&mhZ19});
}

int last_C02;
unsigned long sync_timer = 0;

void loop() {
  if (millis() - sync_timer > 2000) {
    ErrorBitFlags errorFlags;
    ClimateData data;

    data.light = lightSensor.readLightLevel();
    Serial.print("Light Intensity: ");
    Serial.print(data.light);
    Serial.println(" lux");

    data.temperature = bmp280.readTemperature() - 8;
    Serial.print("BMP280 => Temperature = ");
    Serial.print(data.temperature);
    Serial.print(" °C, ");

    data.pressure = bmp280.readPressure() / 100;
    Serial.print("Pressure = ");
    Serial.print(data.pressure / 100);
    Serial.println(" Pa, ");

    data.humidity = SI702x.readHumidity() + 5;
    Serial.print("SI702x => Temperature = ");
    Serial.print(SI702x.readTemperature(), 2);
    Serial.print(" °C, ");
    Serial.print("Humidity = ");
    Serial.println(data.humidity, 2);

    uint16_t errstat, raw; // Read CCS811

    ccs811.set_envdata(data.temperature, data.humidity);
    ccs811.read(&data.eco2, &data.etvoc, &errstat, &raw);
    if (errstat == CCS811_ERRSTAT_OK) {
      Serial.print("CCS811 => CO2 = ");
      Serial.print(data.eco2);
      Serial.print("ppm, TVOC = ");
      Serial.println(data.etvoc);
    } else {
      errorFlags.ccs811 = errstat;
      Serial.print("CCS811 Error: ");
      Serial.println(errstat);
    }

    int CO2 = mhZ19.getCO2();

    if (mhZ19.errorCode == RESULT_OK && CO2 >= 400) {
      Serial.print("CO2 (ppm): ");
      Serial.println(CO2);

      data.co2 = CO2;
      last_C02 = CO2;
    } else if (mhZ19.errorCode == RESULT_OK && CO2 < 300) {
      data.co2 = 400;
      mhZ19.calibrate();
    } else {
      data.co2 = last_C02;
      errorFlags.mhz19 = (ERRORCODE)mhZ19.errorCode;

      Serial.print("MH-Z19 Error: ");
      Serial.println(mhZ19.errorCode);
    }

    bleProtocol.notify(&data, &errorFlags);
    sync_timer = millis();
  }
}