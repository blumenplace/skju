#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

/**
 * P-Wave speed km/S
 */
#define P_WAVE 6.0

/**
 * S-Wave speed km/S
 */
#define S_WAVE 3.5

typedef uint64_t SensorId;

typedef uint64_t Timestamp;

typedef int32_t ReadingValue;

typedef struct Event {
  SensorId sensor_id;
  Timestamp ts;
  ReadingValue gyro_x;
  ReadingValue gyro_y;
  ReadingValue gyro_z;
  ReadingValue accel_x;
  ReadingValue accel_y;
  ReadingValue accel_z;
} Event;
