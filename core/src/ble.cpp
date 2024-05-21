#include "BLECharacteristic.h"
#include "serialize.h"
#include <BLE2902.h>
#include <BLEDevice.h>
#include <BLEServer.h>
#include <BLEUtils.h>

struct BleState {
  bool hasBleConnection = false;
};

class BleServerCallbacks : public BLEServerCallbacks {
  BleState *bleState;

public:
  BleServerCallbacks(BleState *pstate) { this->bleState = pstate; };

  void onConnect(BLEServer *pServer) {
    this->bleState->hasBleConnection = true;
    Serial.println("***** Connect");
  }

  void onDisconnect(BLEServer *pServer) {
    Serial.println("***** Disconnect");
    this->bleState->hasBleConnection = false;
    pServer->getAdvertising()->start();
  }
};

class BleActionCallbacks : public BLECharacteristicCallbacks {
  pSensors *sensors;

public:
  BleActionCallbacks(pSensors *psensors) { this->sensors = psensors; };

  void onWrite(BLECharacteristic *pCharacteristic) {
    std::string value = pCharacteristic->getValue();

    if (value == "CalibrateMhZ19") {
      this->sensors->mhz19->calibrate();
    }
  }
};

class BleProtocol {
  BLEServer *pServer;
  BLECharacteristic *dataCharacteristic;

public:
  BleState bleState;
  pSensors sensors;

public:
  BleProtocol() {}
  void setup(pSensors sensors) {
    this->sensors = sensors;
    // Create the BLE Device
    BLEDevice::init("CO2CICKA Sensor");

    this->pServer = BLEDevice::createServer();

    pServer->setCallbacks(new BleServerCallbacks(&bleState));
    BLEService *pService =
        pServer->createService(BLEUUID("0000FFE0-0000-1000-8000-00805F9B34FB"));

    // Create a BLE Characteristic
    this->dataCharacteristic = pService->createCharacteristic(
        BLEUUID("0000FFE1-0000-1000-8000-00805F9B34FB"),
        BLECharacteristic::PROPERTY_READ | BLECharacteristic::PROPERTY_NOTIFY);
    this->dataCharacteristic->addDescriptor(new BLE2902());

    BLECharacteristic *pCharacteristic = pService->createCharacteristic(
        BLEUUID("beb5483e-36e1-4688-b7f5-ea07361b26a8"),
        BLECharacteristic::PROPERTY_READ | BLECharacteristic::PROPERTY_WRITE);

    pCharacteristic->setCallbacks(new BleActionCallbacks(&sensors));

    pService->start();

    // Start advertising
    BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
    pAdvertising->addServiceUUID(
        BLEUUID("0000FFE0-0000-1000-8000-00805F9B34FB"));
    pAdvertising->start();
  };

  void notify(ClimateData *data, ErrorBitFlags *errorFlags) {
    if (this->bleState.hasBleConnection) {
      this->dataCharacteristic->setValue(
          serializeClimateData(data, errorFlags));
      this->dataCharacteristic->notify();
    }
  }
};
