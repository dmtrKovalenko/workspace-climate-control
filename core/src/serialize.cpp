#include "serialize.h"
#include "ccs811.h"

uint32_t encodeTO2Errors(ErrorBitFlags *errorFlags) {
  uint32_t errorCode = 0;
  if (errorFlags->mhz19 != RESULT_OK) {
    errorCode |= 1 << 0;
  }
  if (errorFlags->bmp280) {
    errorCode |= 1 << 1;
  }
  if (errorFlags->bh1750) {
    errorCode |= 1 << 2;
  }
  if (errorFlags->ccs811 != CCS811_ERRSTAT_OK) {
    errorCode |= 1 << 3;
  }

  return errorCode;
}

#ifdef SERIALIZE_JSON
#include <FirebaseJson.h>

const char *serializeClimateData(ClimateData *data, ErrorBitFlags *errorFlags) {
  FirebaseJson json;
  json.set("temperature", data->temperature);
  json.set("pressure", data->pressure);
  json.set("humidity", data->humidity);
  json.set("light", data->light);
  json.set("co2", data->co2);
  json.set("eco2", data->eco2);
  json.set("etvoc", data->etvoc);

  uint32_t errors = encodeTO2Errors(errorFlags);
  if (errors > 0) {
    json.set("error_flags", errors);
  }

  json.toString(Serial, true);
  return json.raw();
}
#endif