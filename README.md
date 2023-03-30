# DSMR5
A no-std Rust implementation of the DSMR5/4.2 P1 companion standard.

## Intended application
In the first instance using an FTDI -> USB module the smart meter emits its status every second.

## How to use
Plug an FTDI cable into your P1 port. Your RX line will probably need to be inverted. Most off-the-shelf P1 FTDI's already have this preconfigured.
Then create your own little crate using something like the `serial` crate, and use the `Reader` class like so:

```
let mut port = serial::open(&path).unwrap();
let reader = dsmr5::Reader::new(port.bytes().map(|b| b.unwrap()));

for readout in reader {
    let telegram = readout.to_telegram().unwrap();
    let state = dsmr5::Result::<dsmr5::state::State>::from(&telegram).unwrap();

    println!("{}", state.power_delivered.unwrap());
}
```

## References
* [extended Multi-Utility Companion Specification for the Consumer Interface P1 1.7.1](https://maakjemeterslim.be/rails/active_storage/disk/eyJfcmFpbHMiOnsibWVzc2FnZSI6IkJBaDdDRG9JYTJWNVNTSWhlblJ4YW14cU5XdHRhalZsWkRWbmJIaDNNak4zYmpReGFUazRkd1k2QmtWVU9oQmthWE53YjNOcGRHbHZia2tpVjJsdWJHbHVaVHNnWm1sc1pXNWhiV1U5SW1VdFRWVkRVMTlRTVY5RlpGOHhYemRmTVM1d1pHWWlPeUJtYVd4bGJtRnRaU285VlZSR0xUZ25KMlV0VFZWRFUxOVFNVjlGWkY4eFh6ZGZNUzV3WkdZR093WlVPaEZqYjI1MFpXNTBYM1I1Y0dWSkloUmhjSEJzYVdOaGRHbHZiaTl3WkdZR093WlUiLCJleHAiOiIyMDIzLTAyLTI1VDEyOjM4OjA5LjQ5NFoiLCJwdXIiOiJibG9iX2tleSJ9fQ==--3ac24b08420084bc8036a9829c46b503f952a940/e-MUCS_P1_Ed_1_7_1.pdf?content_type=application%2Fpdf&disposition=inline%3B+filename%3D%22e-MUCS_P1_Ed_1_7_1.pdf%22%3B+filename%2A%3DUTF-8%27%27e-MUCS_P1_Ed_1_7_1.pdf)
* [P1 Companion Standard Dutch Smart Meter Requirements 5.0.2 (2016-02-26)](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_a727fce1f1.pdf)
* [P1 Companion Standard Dutch Smart Meter Requirements 4.2.2 (2014-03-14)](https://www.netbeheernederland.nl/_upload/Files/Slimme_meter_15_32ffe3cc38.pdf)
