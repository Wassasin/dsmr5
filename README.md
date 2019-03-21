# DSMR5
A no-std Rust implementation of the DSMR5 standard.

**This implementation is a work in progress and not usable yet.**

## Intended application
In the first instance using an FTDI -> USB module the smart meter emits its status every second.
Later I will be using a custom ARM embedded bord to transmit the status over Zigbee.

## References
* [P1 Companion Standard Dutch Smart Meter Requirements 5.0.2](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_a727fce1f1.pdf)
