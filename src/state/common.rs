use serde::{Deserialize, Serialize};

use crate::{obis::dsmr4::*, types::*, Result};

/// A reading from a power meter, per Tariff.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct MeterReading {
    pub to: Option<f64>,
    pub by: Option<f64>,
}

/// One of three possible lines in the meter.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub voltage_sags: Option<u64>,
    pub voltage_swells: Option<u64>,
    pub voltage: Option<f64>,
    pub active_power_plus: Option<f64>,
    pub active_power_neg: Option<f64>,
    pub current: Option<u64>,
}

/// One of 4 possible slaves to the meter.
///
/// Such as a gas meter, water meter or heat supply.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Slave {
    pub device_type: Option<u64>,
    pub meter_reading: Option<(TST, f64)>,
}

/// The metering state surmised for a single Telegram.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct State {
    pub datetime: Option<TST>,
    pub meterreadings: [MeterReading; 2],
    pub tariff_indicator: Option<[u8; 2]>,
    pub power_delivered: Option<f64>,
    pub power_received: Option<f64>,
    pub power_failures: Option<u64>,
    pub long_power_failures: Option<u64>,
}

impl State {
    pub fn apply(&mut self, o: OBIS) -> Result<()> {
        use OBIS::*;
        match o {
            DateTime(tst) => {
                self.datetime = Some(tst);
            }
            MeterReadingTo(t, mr) => {
                self.meterreadings[t as usize].to = Some(f64::from(&mr));
            }
            MeterReadingBy(t, mr) => {
                self.meterreadings[t as usize].by = Some(f64::from(&mr));
            }
            TariffIndicator(ti) => {
                let mut buf = [0u8; 2];
                let mut octets = ti.as_octets();
                buf[0] = octets.next().unwrap_or(Err(crate::Error::InvalidFormat))?;
                buf[1] = octets.next().unwrap_or(Err(crate::Error::InvalidFormat))?;

                self.tariff_indicator = Some(buf);
            }
            PowerDelivered(p) => {
                self.power_delivered = Some(f64::from(&p));
            }
            PowerReceived(p) => {
                self.power_received = Some(f64::from(&p));
            }
            PowerFailures(UFixedInteger(pf)) => {
                self.power_failures = Some(pf);
            }
            LongPowerFailures(UFixedInteger(lpf)) => {
                self.long_power_failures = Some(lpf);
            }
            _ => {} // Ignore rest.
        }
        Ok(())
    }
}

impl<'a> core::convert::From<&crate::Telegram<'a>> for crate::Result<State> {
    fn from(t: &crate::Telegram<'a>) -> Self {
        t.objects().try_fold(State::default(), |mut state, o| {
            state.apply(o?)?;
            Ok(state)
        })
    }
}
