using System;
using System.Linq;
using System.Net;
using System.Net.Sockets;
using System.Collections.Generic;
using System.Reflection;

using Antmicro.Renode.Core;
using Antmicro.Renode.Logging;
using Antmicro.Renode.Peripherals.Bus;
using Antmicro.Renode.Peripherals.CPU;
using Antmicro.Renode.Utilities;
using Antmicro.Renode.UserInterface;
using Antmicro.Renode.UserInterface.Commands;
using Antmicro.Renode.UserInterface.Tokenizer;
using Antmicro.Renode.Plugins;
using AntShell.Commands;


// Saddenly, we have to use `Antmicro` namespace, otherwise Renode will find the plugin but will fail to load it.
// TODO: Perhaps, we can somehow manipulate the assembly name or AppDomain?
// something like: import clr; clr.AddReference([a for a in System.AppDomain.CurrentDomain.GetAssemblies() if a.GetType('Blumen.Place.DefmtPrint') is not None][0]);
namespace Antmicro.Renode.BlumenPlace
{
    [Plugin(Name = "defmt", Description = "defmt RTT interceptor", Version = "0.1", Vendor = "blumen.place")]
    public class DefmtPlugin: IDisposable
    {
        public DefmtPlugin(Monitor monitor)
        {
            Logger.Log(LogLevel.Info, "[defmt] Defmt plugin loaded, registering 'defmt_setup' command...");
            this.monitor = monitor;
            defmtSetupCommand = new DefmtSetupCommand(monitor);
            monitor.RegisterCommand(defmtSetupCommand);
        }

        public void Dispose()
        {
            defmtSetupCommand.StopServer();
        }

        private readonly DefmtSetupCommand defmtSetupCommand;
        private readonly Monitor monitor;
    }

    public sealed class DefmtSetupCommand: Command
    {
        public DefmtSetupCommand(Monitor monitor):
            base(monitor, "defmt_setup", "Setup defmt RTT interceptor.")
        {
        }

        public override void PrintHelp(ICommandInteraction writer)
        {
            base.PrintHelp(writer);
            writer.WriteLine();
            writer.WriteLine("Usage:");
            writer.WriteLine(String.Format("{0} machine", Name));
        }

        public static object CreateDecoder(string bindingsDllPath, string elfPath)
        {
            var asm = Assembly.LoadFrom(bindingsDllPath);
            var t = asm.GetType("Antmicro.Renode.BlumenPlace.DefmtDecoder", throwOnError: true);
            return Activator.CreateInstance(t!, new object[] { elfPath })!;
        }

        [Runnable]
        public void Run(ICommandInteraction writer, StringToken elfPathToken)
        {
            // TODO: can I automatically inject [AutoParameter] (and for ICPU)?
            var elfPath = elfPathToken.Value;
            var machine = EmulationManager.Instance.CurrentEmulation.Machines.FirstOrDefault();
            Execute(writer, elfPath, machine);
        }

        [Runnable]
        public void Run(ICommandInteraction writer, StringToken elfPathToken, LiteralToken machineToken)
        {
            var elfPath = elfPathToken.Value;
            var machine = (IMachine)monitor.ConvertValueOrThrowRecoverable(machineToken.Value, typeof(Machine));
            Execute(writer, elfPath, machine);
        }

        private void Execute(ICommandInteraction writer, string elfPath, IMachine machine)
        {
            Logger.Log(LogLevel.Debug, "[defmt] Reading SEGGER RTT location");
            var seggerAddress = machine.SystemBus.GetSymbolAddress(SeggerBuffer.SEGGER_RTT);

            Logger.Log(LogLevel.Debug, "[defmt] Initializing decoder");
            dynamic defmtDecoder = CreateDecoder("/opt/renode/DefmtBindings.dll", elfPath);

            var seggerBufferLock = new object();
            SeggerBuffer bufferInfo = null;

            (machine.SystemBus as IBusController).AddWatchpointHook(seggerAddress + SeggerBuffer.WRITE_OFFSET, SysbusAccessWidth.DoubleWord, Access.Write, (cpu, address, width, value) => {
                var data = new byte[0];

                lock (seggerBufferLock) {
                    if (bufferInfo == null)
                    {
                        bufferInfo = new SeggerBuffer(machine, seggerAddress);
                    }
                    data = bufferInfo.ReadUpChannelBuffer((uint)value);
                }

                if (data.Length > 0) {
                    try {
                        var logMessage = defmtDecoder.DecodeFrame(data);
                        if (logMessage != null) {
                            Logger.Log(LogLevel.Info, "[defmt] {0}", logMessage);
                        }
                    } catch (Exception e) {
                        Logger.Log(LogLevel.Error, "[defmt] {0}", e.ToString());
                        throw;
                    }
                    Broadcast(data);
                }
            });

            StartServer();
        }

        private class SeggerBuffer {
            public const string SEGGER_RTT = "_SEGGER_RTT";
            public const ulong BUFFER_PTR_OFFSET = 28;
            public const ulong BUFFER_SIZE_OFFSET = BUFFER_PTR_OFFSET + 4;
            public const ulong WRITE_OFFSET = BUFFER_SIZE_OFFSET + 4;
            public const ulong READ_OFFSET = WRITE_OFFSET + 4;

            private ulong pointer;
            private ulong size;
            private ulong writeAddress;
            private ulong readAddress;
            private IBusController bus;

            public SeggerBuffer(IMachine machine, ulong seggerBase) {
                /*
                defmt declares the SEGGER RTT buffer as:

                #[repr(C)]
                struct Header {
                    id: [u8; 16],
                    max_up_channels: usize,
                    max_down_channels: usize,
                    up_channel: Channel,
                }
                #[repr(C)]
                pub(crate) struct Channel {
                    pub name: *const u8,
                    pub buffer: *mut u8,
                    pub size: usize,
                    pub write: AtomicUsize,
                    pub read: AtomicUsize,
                    pub flags: AtomicUsize,
                }

                Offset calculation:
                ID (16) + MaxUp (4) + MaxDown (4) = 24 bytes
                UpChannel: Name (4), Buffer (4) <- Target is at offset 24 + 4 = 28
                */

                this.bus = machine.SystemBus as IBusController;
                this.writeAddress = seggerBase + WRITE_OFFSET;
                this.readAddress = seggerBase + READ_OFFSET;
                this.pointer = machine.SystemBus.ReadDoubleWord(seggerBase + BUFFER_PTR_OFFSET);
                this.size = machine.SystemBus.ReadDoubleWord(seggerBase + BUFFER_SIZE_OFFSET);
            }

            public byte[] ReadUpChannelBuffer(uint writeValue)
            {
                var readValue = this.bus.ReadDoubleWord(this.readAddress);

                byte[] frameData = new byte[0];
                if (readValue == writeValue) return frameData;

                Logger.Log(LogLevel.Debug, "[defmt] Reading r:{0} w:{1} bytes from defmt buffer", readValue, writeValue);

                if (writeValue >= readValue) {
                    var length = writeValue - readValue;
                    frameData = this.bus.ReadBytes(this.pointer + readValue, (int)length);
                } else {
                    var length = this.size - readValue;
                    var first = this.bus.ReadBytes(this.pointer + readValue, (int)length);

                    if (writeValue > 0) {
                        var second = this.bus.ReadBytes(this.pointer, (int)writeValue);

                        frameData = new byte[first.Length + second.Length];
                        Buffer.BlockCopy(first, 0, frameData, 0, first.Length);
                        Buffer.BlockCopy(second, 0, frameData, first.Length, second.Length);
                    } else {
                        frameData = first;
                    }
                }

                this.bus.WriteDoubleWord(this.readAddress, (uint)writeValue);

                return frameData;
            }
        }

        private void StartServer(int port = 19021)
        {
            Logger.Log(LogLevel.Debug, "[defmt] Starting defmt TCP server on port {0}", port);

            if (isRunning) {
                Logger.Log(LogLevel.Debug, "[defmt] The defmt TCP is already running on port {0}", port);
                return;
            }

            try {
                listener = new TcpListener(IPAddress.Any, port);
                listener.Start();
                isRunning = true;

                var serverThread = new System.Threading.Thread(() => {
                    while (isRunning) {
                        try {
                            Logger.Log(LogLevel.Info, "[defmt] defmt TCP servier is waiting for connections...");
                            var client = listener.AcceptTcpClient();
                            Logger.Log(LogLevel.Info, "[defmt] A client {0} has connected to defmt TCP server", client.Client.RemoteEndPoint);
                            lock (clientsLock) {
                                clients.Add(client);
                            }
                            Logger.Log(LogLevel.Info, "[defmt] Client connected from {0}", client.Client.RemoteEndPoint);
                        } catch {
                            if (isRunning) throw;
                        }
                    }
                }) { IsBackground = true };
                serverThread.Start();
                Logger.Log(LogLevel.Info, "[defmt] TCP server started on port {0}", port);
            } catch (Exception e) {
                Logger.Log(LogLevel.Error, "[defmt] Failed to start Defmt TCP server: {0}", e.Message);
            }
        }

        private void Broadcast(byte[] data)
        {
            lock (clientsLock) {
                Logger.Log(LogLevel.Debug, "[defmt] Broadcasting {0} bytes to defmt TCP clients {1}", data.Length, clients.Count);
                for (int i = clients.Count - 1; i >= 0; i--) {
                    try {
                        var stream = clients[i].GetStream();
                        stream.Write(data, 0, data.Length);
                    } catch {
                        clients[i].Close();
                        clients.RemoveAt(i);
                    }
                }
            }
        }

        public void StopServer()
        {
            isRunning = false;
            listener?.Stop();
            lock(clientsLock)
            {
                foreach(var client in clients) client.Close();
                clients.Clear();
            }
        }

        private TcpListener listener;
        private readonly List<TcpClient> clients = new List<TcpClient>();
        private readonly object clientsLock = new object();
        private bool isRunning;
    }
}
