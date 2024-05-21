#include "data.h"

#define SERIALIZE_JSON

uint32_t encodeTO2Errors(ErrorBitFlags *errorFlags);

const char *serializeClimateData(ClimateData *data, ErrorBitFlags *errorFlags);
