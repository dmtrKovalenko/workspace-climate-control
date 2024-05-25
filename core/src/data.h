#include <MHZ19.h>

struct ClimateData {
  float temperature;
  float pressure;
  float humidity;
  float light;
  int co2;
  uint16_t eco2;
  uint16_t etvoc;
};

struct pSensors {
  MHZ19 *mhz19;
};

struct ErrorBitFlags {
  ERRORCODE mhz19;
  bool bmp280 = false;
  bool bh1750  = false;
  uint16_t ccs811;
};