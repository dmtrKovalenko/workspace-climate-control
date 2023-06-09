#include "Adafruit_VL53L0X.h"

Adafruit_VL53L0X lox = Adafruit_VL53L0X();
#define LEDG 3
void setup()
{
  Serial.begin(115200);
  pinMode(LED_BUILTIN, OUTPUT);
  pinMode(LEDG, OUTPUT);

  Serial.println("Adafruit VL53L0X test");
  Serial.println(LED_BUILTIN);
  if (!lox.begin())
  {
    Serial.println(F("Failed to boot VL53L0X"));
    while (1)
      ;
  }
  // power
  Serial.println(F("VL53L0X API Simple Ranging example\n\n"));
}

void loop()
{
  VL53L0X_RangingMeasurementData_t measure;

  Serial.print("Reading a measurement... ");
  lox.rangingTest(&measure, false); // pass in 'trz mue' to get debug data printout!

  if (measure.RangeStatus != 4)
  { // phase failures have incorrect data
    Serial.print("Distance (mm): ");
    Serial.println(measure.RangeMilliMeter);

    if (measure.RangeMilliMeter < 100)
    {
      digitalWrite(LED_BUILTIN, LOW);
      digitalWrite(LEDG, HIGH);
    }
    else
    {
      digitalWrite(LED_BUILTIN, HIGH);
      digitalWrite(LEDG, LOW);
    }
  }
  else
  {
    digitalWrite(LED_BUILTIN, HIGH);
    digitalWrite(LEDG, LOW);
    Serial.println(" out of range ");
  }

  delay(100);
}