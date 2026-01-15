/**
 * Copyright (c) 2026 Skju Developers under MIT license.
 *
 * [MPU6500 description](https://invensense.tdk.com/wp-content/uploads/2020/06/PS-MPU-6500A-01-v1.3.pdf)
 * [Register Map and Description](https://invensense.tdk.com/wp-content/uploads/2015/02/MPU-6500-Register-Map2.pdf)
 */
using System;
using System.Collections.Generic;
using Antmicro.Renode.Core;
using Antmicro.Renode.Core.Structure.Registers;
using Antmicro.Renode.Logging;
using Antmicro.Renode.Peripherals.Bus;
using Antmicro.Renode.Peripherals.SPI;
using Antmicro.Renode.Peripherals.Sensor;
using Antmicro.Renode.Utilities;


namespace Antmicro.Renode.Peripherals.Sensors
{
    public class Mpu6500 : ISPIPeripheral, IProvidesRegisterCollection<ByteRegisterCollection>, ISensor
    {
        public Mpu6500()
        {
            RegistersCollection = new ByteRegisterCollection(this);
            random = EmulationManager.Instance.CurrentEmulation.RandomGenerator;
            
            DefineRegisters();
            Reset();
        }

        public void Reset()
        {
            RegistersCollection.Reset();
            registers[(int)Registers.WhoAmI] = MPU6500;
            registers[(int)Registers.PowerManagement1] = 0x41; // Default power management state
        }

        public byte Transmit(byte data)
        {
            if (isFirstByte)
            {
                // First byte is the register address
                currentRegister = (byte)(data & 0x7F); // Remove R/W bit
                isReadOperation = (data & 0x80) != 0;
                isFirstByte = false;
                
                this.Log(LogLevel.Debug, "SPI operation: {0} register {1}", 
                    isReadOperation ? "Read" : "Write", (Registers)currentRegister);
                
                return 0x00; // Dummy response for address byte
            }
            else
            {
                if (isReadOperation)
                {
                    // Read operation - return register value
                    byte value = ReadRegister(currentRegister);
                    currentRegister++; // Auto-increment for burst reads
                    return value;
                }
                else
                {
                    // Write operation - store data to register
                    WriteRegister(currentRegister, data);
                    currentRegister++; // Auto-increment for burst writes
                    return 0x00; // Dummy response
                }
            }
        }

        public void FinishTransmission()
        {
            isFirstByte = true;
            this.Log(LogLevel.Debug, "SPI transmission finished");
        }

        public ByteRegisterCollection RegistersCollection { get; }

        private void DefineRegisters()
        {
            // Initialize register array
            registers = new byte[128];
            
            // Register to indicate to user which device is being accessed.
            Registers.WhoAmI.Define(this)
                .WithValueField(0, 8, FieldMode.Read, valueProviderCallback: _ => MPU6500, name: "WHO_AM_I");

            Registers.GyroConfiguration.Define(this)
            /*
                .WithFlag(7, FieldMode.Read | FieldMode.Write, name: "X Gyro self-test", ) // XG_ST
                .WithFlag(6, FieldMode.Read | FieldMode.Write, name: "Y Gyro self-test", ) // YG_ST
                .WithFlag(5, FieldMode.Read | FieldMode.Write, name: "Z Gyro self-test", ) // ZG_ST
                .WithFlag(4,)
                .WithEnumField(0, 3, out responseTypeField, name: "respone_type")
                .WithReservedBits(3, 2)
                            .WithEnumField<DoubleWordRegister, TransferType>(5, 3, out transferTypeField, name: "transfer_type")
                            .WithIgnoredBits(8, 24);*/


                .WithValueField(0, 8, FieldMode.Read | FieldMode.Write, name: "GYRO_CONFIG");

                
            Registers.PowerManagement1.Define(this)
                .WithValueField(0, 8, FieldMode.Read | FieldMode.Write, name: "PWR_MGMT_1");
                
            Registers.PowerManagement2.Define(this)
                .WithValueField(0, 8, FieldMode.Read | FieldMode.Write, name: "PWR_MGMT_2");
                
            Registers.SampleRateDivider.Define(this)
                .WithValueField(0, 8, FieldMode.Read | FieldMode.Write, name: "SMPLRT_DIV");
                
            Registers.Configuration.Define(this)
                .WithValueField(0, 8, FieldMode.Read | FieldMode.Write, name: "CONFIG");
                
            Registers.AccelConfiguration.Define(this)
                .WithValueField(0, 8, FieldMode.Read | FieldMode.Write, name: "ACCEL_CONFIG");
        }

        private byte ReadRegister(byte address)
        {
            switch (address)
            {
                case var addr when addr >= 0x3B && addr <= 0x40: // Accelerometer data
                    return GenerateAccelData(addr);
                    
                case var addr when addr >= 0x41 && addr <= 0x42: // Temperature data
                    return GenerateTemperatureData(addr);
                    
                case var addr when addr >= 0x43 && addr <= 0x48: // Gyroscope data
                    return GenerateGyroData(addr);
                    
                default:
                    if (address < registers.Length)
                    {
                        return registers[address];
                    }
                    this.Log(LogLevel.Warning, "Reading from undefined register {0}", (Registers)address);
                    return 0x00;
            }
        }

        private void WriteRegister(byte address, byte value)
        {
            if (address < registers.Length)
            {
                registers[address] = value;
                this.Log(LogLevel.Debug, "Write to register {0}: 0x{1:X2}", (Registers)address, value);
            }
            else
            {
                this.Log(LogLevel.Warning, "Writing to undefined register {0}", (Registers)address);
            }
        }

        private byte GenerateAccelData(byte address)
        {
            // Generate random accelerometer data (-2g to +2g range, 16-bit)
            var accelValue = (short)(random.Next(-32768, 32767));
            
            switch ((Registers)address)
            {
                case Registers.AccelXOutHigh: return (byte)(accelValue >> 8);
                case Registers.AccelXOutLow: return (byte)(accelValue & 0xFF);
                case Registers.AccelYOutHigh: return (byte)(accelValue >> 8);
                case Registers.AccelYOutLow: return (byte)(accelValue & 0xFF);
                case Registers.AccelZOutHigh: return (byte)(accelValue >> 8);
                case Registers.AccelZOutLow: return (byte)(accelValue & 0xFF);
                default: return 0x00;
            }
        }

        private byte GenerateTemperatureData(byte address)
        {
            // Generate random temperature data (around 25°C)
            var tempValue = (short)(random.Next(8000, 9000)); // Roughly 25°C in MPU6500 units
            
            switch ((Registers)address)
            {
                case Registers.TempOutHigh: return (byte)(tempValue >> 8);
                case Registers.TempOutLow: return (byte)(tempValue & 0xFF);
                default: return 0x00;
            }
        }

        private byte GenerateGyroData(byte address)
        {
            // Generate random gyroscope data (±250 dps range, 16-bit)
            var gyroValue = (short)(random.Next(-32768, 32767));

            switch ((Registers)address)
            {
                case Registers.GyroXOutHigh: return (byte)(gyroValue >> 8);
                case Registers.GyroXOutLow: return (byte)(gyroValue & 0xFF);
                case Registers.GyroYOutHigh: return (byte)(gyroValue >> 8);
                case Registers.GyroYOutLow: return (byte)(gyroValue & 0xFF);
                case Registers.GyroZOutHigh: return (byte)(gyroValue >> 8);
                case Registers.GyroZOutLow: return (byte)(gyroValue & 0xFF);
                default: return 0x00;
            }
        }

        // It selects the full-scale range of the gyroscope (±dps).
        //
        // GYRO_FS_SEL bits[4:3] of the GYRO_CONFIG register as defined in the MPU-6500 register map,
        // section 4.6, page 14.
        private enum GyroFullScaleSelect {
            Dps250 = 0b00,
            Dps500 = 0b01,
            Dps1000 = 0b10,
            Dps2000 = 0b11
        }

        private enum Registers
        {
            WhoAmI = 0x75,

            SampleRateDivider = 0x19,

            Configuration = 0x1A,

            // Register 27 – Gyroscope Configuration
            GyroConfiguration = 0x1B,

            AccelConfiguration = 0x1C,

            AccelXOutHigh = 0x3B,
            AccelXOutLow = 0x3C,

            AccelYOutHigh = 0x3D,
            AccelYOutLow = 0x3E,

            AccelZOutHigh = 0x3F,
            AccelZOutLow = 0x40,

            TempOutHigh = 0x41,
            TempOutLow = 0x42,

            GyroXOutHigh = 0x43,
            GyroXOutLow = 0x44,

            GyroYOutHigh = 0x45,
            GyroYOutLow = 0x46,

            GyroZOutHigh = 0x47,
            GyroZOutLow = 0x48,

            PowerManagement1 = 0x6B,
            PowerManagement2 = 0x6C,
        }

        private byte[] registers;
        private byte currentRegister;
        private bool isReadOperation;
        private bool isFirstByte = true;
        private readonly PseudorandomNumberGenerator random;

        private const byte MPU6500 = 0x70;
    }
}