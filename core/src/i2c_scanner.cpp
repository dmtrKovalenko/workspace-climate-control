#include <Wire.h>
#include <Arduino.h>

void scan_i2c_devices(TwoWire *wire)
{
  byte error, address;
  int devicesFound = 0;

  Serial.println("Scanning I2C bus...");

  for (address = 1; address < 127; address++)
  {
    wire->beginTransmission(address);
    error = wire->endTransmission();

    if (error == 0)
    {
      Serial.print("Device found at address 0x");
      if (address < 16)
      {
        Serial.print("0");
      }
      Serial.print(address, HEX);
      Serial.println();
      devicesFound++;
    }
  }

  if (devicesFound == 0)
  {
    Serial.println("No devices found.");
  }
}