#include "../../shared/conf.h"
#include "BLEService.h"
#include "calibration.cpp"
#include <BLE2902.h>
#include <BLECharacteristic.h>
#include <BLEDevice.h>
#include <BLEServer.h>
#include <BLEUtils.h>
#include <serialize.h>
#include <stdint.h>

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

class BleC02ActionCallbacks : public BLECharacteristicCallbacks {
  Calibrator *calibration;

public:
  BleC02ActionCallbacks(Calibrator *calibration) {
    this->calibration = calibration;
  };

  void onWrite(BLECharacteristic *pCharacteristic) {
    int result = calibration->calibrate(pCharacteristic->getData(),
                                        pCharacteristic->getLength());
    pCharacteristic->setValue(result);
  }
};

class BleProtocol {
  BLEServer *pServer;
  BLECharacteristic *dataCharacteristic;

  BLECharacteristic *create_calibration_characteristic(BLEService *pService,
                                                       BLEUUID uuid,
                                                       Calibrator *calibrator) {
    BLECharacteristic *pCharacteristic = pService->createCharacteristic(
        uuid,
        BLECharacteristic::PROPERTY_READ | BLECharacteristic::PROPERTY_WRITE);
    pCharacteristic->setCallbacks(new BleC02ActionCallbacks(calibrator));

    int response = calibrator->isCalibrated();
    pCharacteristic->setValue(response);

    return pCharacteristic;
  }

public:
  BleState bleState;
  pSensors sensors;

public:
  BleProtocol() {}
  void setup(Calibration *calibration) {
    this->sensors = sensors;
    // Create the BLE Device
    BLEDevice::init(BLE_MAIN_SERVICE_LOCAL_NAME);

    this->pServer = BLEDevice::createServer();

    pServer->setCallbacks(new BleServerCallbacks(&bleState));
    BLEService *pService =
        pServer->createService(BLEUUID(BLE_MAIN_SENSOR_SERVICE));

    // Create a BLE Characteristic
    this->dataCharacteristic = pService->createCharacteristic(
        BLEUUID(BLE_MAIN_SENSOR_STREAM_CHAR),
        BLECharacteristic::PROPERTY_READ | BLECharacteristic::PROPERTY_NOTIFY |
            BLECharacteristic::PROPERTY_INDICATE);
    this->dataCharacteristic->addDescriptor(new BLE2902());

    BLECharacteristic *co2_char = this->create_calibration_characteristic(
        pService, BLEUUID(BLE_MAIN_SENSOR_CO2_CALIBRATION_CHAR),
        &calibration->mhz19Calibration);

    BLECharacteristic *temp_char = this->create_calibration_characteristic(
        pService, BLEUUID(BLE_MAIN_SENSOR_TEMP_CALIBRATION_CHAR),
        &calibration->temperatureCalibration);

    pService->start();

    // Start advertising
    BLEAdvertising *pAdvertising = BLEDevice::getAdvertising();
    pAdvertising->addServiceUUID(BLEUUID(BLE_MAIN_SENSOR_SERVICE));
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