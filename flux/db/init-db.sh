#!/bin/bash
set -e

clickhouse client -n <<-EOSQL
  CREATE DATABASE IF NOT EXISTS skju;
  CREATE TABLE IF NOT EXISTS skju.events
  (
      sensor_id UInt32,

      ts DateTime64(3, 'Europe/Belgrade'),

      gyro_x Int16,
      gyro_y Int16,
      gyro_z Int16,

      axel_x Int16,
      axel_y Int16,
      axel_z Int16
  )
  ENGINE = MergeTree
  ORDER BY (sensor_id, ts);
EOSQL

#    CREATE USER skju_flux_app IDENTIFIED BY 'XXX';
#    GRANT SELECT ON skju.events TO skju_flux_app;
#    CREATE ROLE reader;
