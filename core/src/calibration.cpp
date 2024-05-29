#include "../../shared/conf.h"
#include "data.h"
#include <Preferences.h>
#include <cstddef>
#include <stdint.h>

class Calibrator {
public:
  pSensors sensors;
  Preferences preferences;

  virtual bool calibrate(uint8_t *value, size_t size) = 0;
  virtual bool isCalibrated() = 0;
  virtual void adjustMeasurement(ClimateData *data) {};
};

class MHZ19Calibration : public Calibrator {
public:
  bool isCalibrated() {
    return this->preferences.getBool("co2_calibrated", false);
  }

  bool calibrate(uint8_t *value, size_t size) {
    Serial.println("Calibrating CO2 sensor");
    this->sensors.mhz19->calibrate();
    this->preferences.putBool("co2_calibrated", true);

    return true;
  }
};

class TemperatureCalibration : public Calibrator {
  int temperature_adjust = CALIBRATION_TEMPERATURE_ADJUST;
  bool deserialize_ble_data(uint8_t *data, size_t dataSize, int32_t &dest) {
    if (data == NULL || dataSize < sizeof(int32_t)) {
      return false; // Error: data is NULL or not enough data to convert
    }

    // Assuming little-endian byte order
    dest =
        (int32_t)(data[0] | (data[1] << 8) | (data[2] << 16) | (data[3] << 24));

    return true;
  }

public:
  TemperatureCalibration() {
    this->temperature_adjust = this->preferences.getInt(
        "bmp280_calibration", CALIBRATION_TEMPERATURE_ADJUST);
  };

  bool isCalibrated() {
    return this->preferences.getInt("bmp280_calibration", -2587) != -2587;
  }

  bool calibrate(uint8_t *value, size_t size) {
    int32_t temp;
    if (!deserialize_ble_data(value, size, temp)) {
      return false;
    }

    this->temperature_adjust = temp;
    this->preferences.putInt("bmp280_calibration", temp);

    return true;
  }
  void adjustMeasurement(ClimateData *data) {
    data->temperature += temperature_adjust;
  }
};

class Calibration {
public:
  MHZ19Calibration mhz19Calibration;
  TemperatureCalibration temperatureCalibration;
  Calibration(pSensors sensors) {
    this->mhz19Calibration.sensors = sensors;
    this->temperatureCalibration.sensors = sensors;
  }

  void adjustMeasurement(ClimateData *data) {
    this->mhz19Calibration.adjustMeasurement(data);
    this->temperatureCalibration.adjustMeasurement(data);
  }
};