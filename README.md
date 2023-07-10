# DSMR5
A no-std Rust implementation of the DSMR5/4.2 P1 companion standard.

## Intended application
In the first instance using an FTDI -> USB module the smart meter emits its status every second.

## How to use
Plug an FTDI cable into your P1 port. Your RX line will probably need to be inverted. Most off-the-shelf P1 FTDI's already have this preconfigured.
Then create your own little crate using something like the `serial` crate, and use the `Reader` class like so:

```
let mut port = serial::open(&path).unwrap();
let reader = dsmr5::Reader::new(port.bytes());

for readout in reader {
    let telegram = readout.unwrap().to_telegram().unwrap();
    let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();

    println!("{}", state.power_delivered.unwrap());
}
```

## References
* [P1 Companion Standard Dutch Smart Meter Requirements 5.0.2 (2016-02-26)](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_a727fce1f1.pdf)
* [P1 Companion Standard Dutch Smart Meter Requirements 4.2.2 (2014-03-14)](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_32ffe3cc38.pdf)
