#include "eventspod.h"

Event event_create(uint64_t id, uint64_t ts) {
    Event event = {0};
    event.sensor_id = id;
    event.ts = ts;
    return event;
}

void event_set_gyro(Event *event, uint16_t x, uint16_t y, uint16_t z) {
    if (event) {
        event->gyro_x = x;
        event->gyro_y = y;
        event->gyro_z = z;
    }
}

void event_set_accel(Event *event, uint16_t x, uint16_t y, uint16_t z) {
    if (event) {
        event->accel_x = x;
        event->accel_y = y;
        event->accel_z = z;
    }
}


