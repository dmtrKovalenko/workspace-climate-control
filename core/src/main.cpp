#include <Wire.h>
#include <Adafruit_Sensor.h>
#include "ccs811.h"          // include library for CCS811 - Sensor from martin-pennings https://github.com/maarten-pennings/CCS811
#include "Adafruit_Si7021.h" // include main library for SI7021 - Sensor
#include "Adafruit_BMP280.h" // include main library for BMP280 - Sensor
#include <BLEDevice.h>
#include <BLEServer.h>
#include <BLEUtils.h>
#include <BLE2902.h>
#include <BH1750.h>
#include "i2c_scanner.h"

// DHT dht(DHTPIN, DHTTYPE);
CCS811 ccs811;
Adafruit_BMP280 bmp280; // I2C
Adafruit_Si7021 SI702x = Adafruit_Si7021();
BH1750 lightSensor(0x23);

BLECharacteristic *pCharacteristic;
bool deviceConnected = false;

class MyServerCallbacks : public BLEServerCallbacks
{
  void onConnect(BLEServer *pServer)
  {
    deviceConnected = true;
    Serial.println("***** Connect");
  }

  void onDisconnect(BLEServer *pServer)
  {
    Serial.println("***** Disconnect");
    deviceConnected = false;
    pServer->getAdvertising()->start();
  }
};

void setup()
{
  Serial.begin(115200);

  Serial.println("DHT22 Temperature and Humidity Sensor");
  Serial.println("------------------------------------");

  Wire.begin(21, 22); // Configure I2C pins (SDA: GPIO 21, SCL: GPIO 22)
  Wire1.begin(26, 25);

  if (!lightSensor.begin(BH1750::CONTINUOUS_HIGH_RES_MODE, 0x23, &Wire1))
  {
    Serial.println("Error initializing BH1750");
  }

  ccs811.set_i2cdelay(50); // Needed for ESP8266 because it doesn't handle I2C clock stretch correctly
  if (!ccs811.begin())
  {
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
  if (!bmp280.begin(0x76))
  {
    Serial.println("Could not find a valid BMP280 sensor, check wiring!");
  }

  Serial.println("Si7021 test!"); /* ---- SETUP SI702x ----- */
  if (!SI702x.begin())
  {
    Serial.println("Did not find Si702x sensor!");
  }

  Serial.print("Found model ");
  switch (SI702x.getModel())
  {
  case SI_Engineering_Samples:
    Serial.print("SI engineering samples");
    break;
  case SI_7013:
    Serial.print("Si7013");
    break;
  case SI_7020:
    Serial.print("Si7020");
    break;
  case SI_7021:
    Serial.print("Si7021");
    break;
  case SI_UNKNOWN:
  default:
    Serial.print("Unknown");
  }
  Serial.print(" Revision(");
  Serial.print(SI702x.getRevision());
  Serial.print(")");
  Serial.print(" Serial #");
  Serial.print(SI702x.sernum_a, HEX);
  Serial.println(SI702x.sernum_b, HEX);

  // Wait for the sensors to stabilize
  delay(2000);

  // Create the BLE Device
  BLEDevice::init("CO2CICKA Sensor");

  BLEServer *pServer = BLEDevice::createServer();
  pServer->setCallbacks(new MyServerCallbacks());
  BLEService *pService = pServer->createService(BLEUUID("0000FFE0-0000-1000-8000-00805F9B34FB"));

  // Create a BLE Characteristic
  pCharacteristic = pService->createCharacteristic(
      BLEUUID("0000FFE1-0000-1000-8000-00805F9B34FB"),
      BLECharacteristic::PROPERTY_READ |
          BLECharacteristic::PROPERTY_NOTIFY);

  pCharacteristic->addDescriptor(new BLE2902());
  pService->start();

  // Start advertising
  BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
  pAdvertising->addServiceUUID(BLEUUID("0000FFE0-0000-1000-8000-00805F9B34FB"));
  pAdvertising->start();
}

void loop()
{
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
  Serial.println(SI702x.readHumidity(), 2);

  uint16_t eco2, etvoc, errstat, raw; // Read CCS811

  ccs811.set_envdata(temperature, humidity);
  ccs811.read(&eco2, &etvoc, &errstat, &raw);
  if (errstat == CCS811_ERRSTAT_OK)
  {
    Serial.print("CCS811 => CO2 = ");
    Serial.print(eco2);
    Serial.print("ppm, TVOC = ");
    Serial.println(etvoc);
  }

  String sensorData = String(eco2) + "," + String(etvoc) + "," + String(temperature) + "," + String(pressure) + "," + String(humidity) + "," + String(lightIntensity);
  pCharacteristic->setValue(sensorData.c_str());
  pCharacteristic->notify();

  delay(5000);
}