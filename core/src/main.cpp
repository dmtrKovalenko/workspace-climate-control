#include "Adafruit_BMP280.h" // include main library for BMP280 - Sensor
#include "Adafruit_Si7021.h" // include main library for SI7021 - Sensor
#include "HardwareSerial.h"
#include "MHZ19.h"
#include "ccs811.h"
#include "i2c_scanner.h"
#include <Adafruit_Sensor.h>
#include <Arduino.h>
#include <BH1750.h>
#include <BLE2902.h>
#include <BLEDevice.h>
#include <BLEServer.h>
#include <BLEUtils.h>
#include <FirebaseJson.h>
#include <Wire.h>

#define BAUDRATE 9600

CCS811 ccs811;
Adafruit_BMP280 bmp280; // I2C
Adafruit_Si7021 SI702x = Adafruit_Si7021();
BH1750 lightSensor(0x23);
MHZ19 myMHZ19;

HardwareSerial mySerial(2);

BLECharacteristic *pCharacteristic;
bool deviceConnected = false;

class MyServerCallbacks : public BLEServerCallbacks {
  void onConnect(BLEServer *pServer) {
    deviceConnected = true;
    Serial.println("***** Connect");
  }

  void onDisconnect(BLEServer *pServer) {
    Serial.println("***** Disconnect");
    deviceConnected = false;
    pServer->getAdvertising()->start();
  }
};

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

  // Wait for the sensors to stabilize delay(2000);

  while (myMHZ19.errorCode != RESULT_OK) {
    Serial.println("Estabilshing connection to MH-Z19");
    myMHZ19.begin(mySerial);
  };

  myMHZ19.calibrate();

  // Create the BLE Device
  BLEDevice::init("CO2CICKA Sensor");

  BLEServer *pServer = BLEDevice::createServer();
  pServer->setCallbacks(new MyServerCallbacks());
  BLEService *pService =
      pServer->createService(BLEUUID("0000FFE0-0000-1000-8000-00805F9B34FB"));

  // Create a BLE Characteristic
  pCharacteristic = pService->createCharacteristic(
      BLEUUID("0000FFE1-0000-1000-8000-00805F9B34FB"),
      BLECharacteristic::PROPERTY_READ | BLECharacteristic::PROPERTY_NOTIFY);

  pCharacteristic->addDescriptor(new BLE2902());
  pService->start();

  // Start advertising
  BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
  pAdvertising->addServiceUUID(BLEUUID("0000FFE0-0000-1000-8000-00805F9B34FB"));
  pAdvertising->start();
}

int last_C02;
unsigned long sync_timer = 0;

void loop() {
  if (millis() - sync_timer > 5000) {
    int CO2 = myMHZ19.getCO2(false);

    float lightIntensity = lightSensor.readLightLevel();
    Serial.print("Light Intensity: ");
    Serial.print(lightIntensity);
    Serial.println(" lux");

    float temperature = bmp280.readTemperature() - 4;
    Serial.print("BMP280 => Temperature = ");
    Serial.print(temperature);
    Serial.print(" °C, ");

    float pressure = bmp280.readPressure() / 100;
    Serial.print("Pressure = ");
    Serial.print(bmp280.readPressure() / 100);
    Serial.println(" Pa, ");

    float humidity = SI702x.readHumidity() + 5;
    Serial.print("SI702x => Temperature = ");
    Serial.print(SI702x.readTemperature(), 2);
    Serial.print(" °C, ");
    Serial.print("Humidity = ");
    Serial.println(humidity, 2);

    uint16_t eco2, etvoc, errstat, raw; // Read CCS811

    ccs811.set_envdata(temperature, humidity);
    ccs811.read(&eco2, &etvoc, &errstat, &raw);
    if (errstat == CCS811_ERRSTAT_OK) {
      Serial.print("CCS811 => CO2 = ");
      Serial.print(eco2);
      Serial.print("ppm, TVOC = ");
      Serial.println(etvoc);
    }

    FirebaseJson json;
    if (myMHZ19.errorCode == RESULT_OK && CO2 > 0) {
      Serial.print("CO2 (ppm): ");
      Serial.println(CO2);
      /*Serial.print("MHZ19 => Temperature (C): ");*/
      /*Serial.println(Temp);*/

      json.add("co2", CO2);
      last_C02 = CO2;
    } else {
      json.add("co2", last_C02);
      json.add("mhz19_error_code", myMHZ19.errorCode);

      Serial.print("MH-Z19 Error: ");
      Serial.println(myMHZ19.errorCode);
    }

    if (lightIntensity > 0) {
      json.add("light", lightIntensity);
    }

    if (temperature && pressure && humidity) {
      json.add("temperature", temperature);
      json.add("pressure", pressure);
      json.add("humidity", humidity);
    }

    json.add("eco2", eco2);
    json.add("etvoc", etvoc);

    json.toString(Serial, true);

    pCharacteristic->setValue(json.raw());
    pCharacteristic->notify();

    sync_timer = millis();
  }
}