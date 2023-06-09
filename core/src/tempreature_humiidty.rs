#include <Adafruit_Sensor.h>
#include <DHT.h>
#include <DHT_U.h>
#include <Wire.h>
#include <SPI.h>

#define DHTPIN 21     // GPIO pin connected to the DHT22 data pin
#define DHTTYPE DHT22 // DHT sensor type (DHT11 or DHT22)

DHT_Unified dht(DHTPIN, DHTTYPE);

void setup()
{
  Serial.begin(115200);
  dht.begin();
  sensor_t sensor;
  dht.temperature().getSensor(&sensor);
  Serial.println("DHT22 Temperature and Humidity Sensor");
  Serial.println("------------------------------------");
  Serial.print("Sensor: ");
  Serial.println(sensor.name);
  Serial.print("Driver Ver: ");
  Serial.println(sensor.version);
  Serial.print("Unique ID: ");
  Serial.println(sensor.sensor_id);
  Serial.print("Max Value: ");
  Serial.print(sensor.max_value);
  Serial.println("°C");
  Serial.print("Min Value: ");
  Serial.print(sensor.min_value);
  Serial.println("°C");
  Serial.print("Resolution: ");
  Serial.print(sensor.resolution);
  Serial.println("°C");
  Serial.println("------------------------------------");
  delay(2000); // Allow the sensor to stabilize
}

void loop()
{
  sensors_event_t event;
  dht.temperature().getEvent(&event);
  if (isnan(event.temperature))
  {
    Serial.println("Error reading temperature!");
  }
  else
  {
    Serial.print("Temperature: ");
    Serial.print(event.temperature);
    Serial.println("°C");
  }
  dht.humidity().getEvent(&event);
  if (isnan(event.relative_humidity))
  {
    Serial.println("Error reading humidity!");
  }
  else
  {
    Serial.print("Humidity: ");
    Serial.print(event.relative_humidity);
    Serial.println("%");
  }
  delay(2000); // Delay between readings
}