#import "@preview/ilm:2.0.0": *
#import "@preview/gentle-clues:1.3.1": *
#import "@preview/chronos:0.3.0"

// #import "@preview/cetz:0.4.2"
// #import "@preview/zero:0.6.1": num, format-table, zi

// #import "@preview/quick-maths:0.2.1": shorthands
// #import "@preview/lilaq:0.6.0" as lq

// #import "@preview/physica:0.9.8": *
// #import "@preview/pavemat:0.2.0": pavemat
// #import "@preview/zebraw:0.6.1": *

#set text(lang: "en")
#show figure.caption: emph

#show: ilm.with(
  title: [Skju - Seismic Stations Network],
  authors: "todo",
  date: datetime(year: 2026, month: 04, day: 2),
  abstract: [
    Analysis of time-series data boils down to understanding
    how events that happened in the part will affect future events. @prac
  ],
  bibliography: bibliography("refs.bib"),
  figure-index: (enabled: true),
  table-index: (enabled: true),
  listing-index: (enabled: true)
)


= Sensors and Seismic Stations Identification

Reliable node identification is a foundational requirement in distributed sensor networks,
underpinning message routing, data provenance, fault isolation, and deduplication of telemetry.
In safety-critical contexts such as seismic monitoring, identifier integrity
directly affects the reliability of event attribution.

The Skju system comprises two classes of hardware nodes:
- field sensors built on the Nordic Semiconductor nRF52 series
- and seismic stations built on the nRF91 series.

To uniquely identify each node across the network, we evaluated
- auto-increment integer counters,
- UUIDv4,
- and IMEI printed by Nordic on the nrf9160dk product’s label
all of which would have required identifiers to be programmed into hardware during production.

We ultimately adopted the hardware-native DEVICEID register available on Nordic Semiconductor chips.

Each DEVICEID value is generated randomly during chip production using a FIPS-approved random
number generator, yielding a per-pair collision probability of $(5.42 times 10^(-20))$ effectively
negligible at the scale of any foreseeable deployment.
Therefore DEVICEID can be reliably used as MQTT client ID.

= Skatch

+ Depending on the region of Earth you are living in
earth quake can be serios problem
causing (TODO: provide a reference) money in damages
of infrastructure
health expenses
and expenses non recoverable - human death and casualities.

According to ... 


= Electrical Characteristis

Current consumption @ 3.7 V:
- Power saving mode (PSM) floor current: 2.7 µA
- eDRX @ 81.92s: 18 µA in Cat-M1, 37 µA in Cat-NB1 (UICC included)
@nrf9160

= Naive Earth Quake Detection


multisite systems at low cost with low latency


We successfully developed a low-latency, Bluetooth low energy (BLE)-based data alignment method,
implemented in the BLE application layer, making it transferable between manufacturer devices. The time synchronization method was tested on two commercial BLE platforms by inputting common sinusoidal input signals (over a range of frequencies) to evaluate time alignment performance between two independent peripheral nodes. 




= Time Sychronization in Sensors's Network

+ Describe the motivation for time synchronization.
+ Describe the prior art @tmsens.
+ Describe the approach we took.

= Heim Service

Heim Service is ment to be an interal service and a source of truth
for Seismic Stations' metadata.

#figure(
    chronos.diagram({
      import chronos: *
      _par("Alice")
      _par("Bob")
    }),
  caption: [A curious figure.],
) <glacier>



Yet, there is one exception --- Simulator iPadOS app.
The app is allow to read and write to Heim from outside of protected perimeter.


#code(title: "Protol Buffer Definition")[```protobuf

// some protocol buffers definition

```]
