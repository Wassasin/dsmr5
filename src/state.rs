//! Convenience structs to get and keep the current state of the meter in memory.
//!
//! Usage of these types is entirely optional.
//! When only needing a few or single record, it is more efficient to directly filter on the
//! telegram objects iterator.

use serde::{Deserialize, Serialize};

use crate::{
    obis::{dsmr4::*, *},
    types::*,
};

/// A reading from a power meter, per Tariff.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct MeterReading {
    pub to: Option<f64>,
    pub by: Option<f64>,
}

/// One of three possible lines in the meter.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Line {
    pub voltage_sags: Option<u64>,
    pub voltage_swells: Option<u64>,
    pub voltage: Option<f64>,
    #[cfg(not(feature = "emucs"))]
    pub current: Option<u64>,
    #[cfg(feature = "emucs")]
    pub current: Option<f64>,
    pub active_power_plus: Option<f64>,
    pub active_power_neg: Option<f64>,
}

/// One of 4 possible slaves to the meter.
///
/// Such as a gas meter, water meter or heat supply.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Slave {
    pub device_type: Option<u64>,
    pub meter_reading: Option<(TST, f64)>,
    #[cfg(feature = "emucs")]
    pub valve_state: Option<u64>,
}

/// The metering state surmised for a single Telegram.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct State {
    pub datetime: Option<TST>,
    pub meterreadings: [MeterReading; 2],
    pub tariff_indicator: Option<[u8; 2]>,
    pub power_delivered: Option<f64>,
    pub power_received: Option<f64>,
    pub power_failures: Option<u64>,
    pub long_power_failures: Option<u64>,
    #[cfg(feature = "emucs")]
    pub breaker_state: Option<u64>,
    #[cfg(feature = "emucs")]
    pub limiter_threshold: Option<f64>,
    #[cfg(feature = "emucs")]
    pub fuse_supervision_threshold: Option<u64>,
    pub lines: [Line; 3],
    pub slaves: [Slave; 4],
}

impl<'a> core::convert::From<&crate::Telegram<'a>> for crate::Result<State> {
    fn from(t: &crate::Telegram<'a>) -> Self {
        t.objects().try_fold(State::default(), |mut state, o| {
            match o? {
                OBIS::DateTime(tst) => {
                    state.datetime = Some(tst);
                }
                OBIS::MeterReadingTo(t, mr) => {
                    state.meterreadings[t as usize].to = Some(f64::from(&mr));
                }
                OBIS::MeterReadingBy(t, mr) => {
                    state.meterreadings[t as usize].by = Some(f64::from(&mr));
                }
                OBIS::TariffIndicator(ti) => {
                    let mut buf = [0u8; 2];
                    let mut octets = ti.as_octets();
                    buf[0] = octets.next().unwrap_or(Err(crate::Error::InvalidFormat))?;
                    buf[1] = octets.next().unwrap_or(Err(crate::Error::InvalidFormat))?;

                    state.tariff_indicator = Some(buf);
                }
                OBIS::PowerDelivered(p) => {
                    state.power_delivered = Some(f64::from(&p));
                }
                OBIS::PowerReceived(p) => {
                    state.power_received = Some(f64::from(&p));
                }
                OBIS::PowerFailures(UFixedInteger(pf)) => {
                    state.power_failures = Some(pf);
                }
                OBIS::LongPowerFailures(UFixedInteger(lpf)) => {
                    state.long_power_failures = Some(lpf);
                }
                #[cfg(feature = "emucs")]
                OBIS::BreakerState(UFixedInteger(b)) => {
                    state.breaker_state = Some(b);
                }
                #[cfg(feature = "emucs")]
                OBIS::LimiterThreshold(l) => {
                    state.limiter_threshold = Some(f64::from(&l));
                }
                #[cfg(feature = "emucs")]
                OBIS::FuseSupervisionThreshold(UFixedInteger(f)) => {
                    state.fuse_supervision_threshold = Some(f);
                }
                OBIS::VoltageSags(l, UFixedInteger(n)) => {
                    state.lines[l as usize].voltage_sags = Some(n);
                }
                OBIS::VoltageSwells(l, UFixedInteger(n)) => {
                    state.lines[l as usize].voltage_swells = Some(n);
                }
                OBIS::InstantaneousVoltage(l, v) => {
                    state.lines[l as usize].voltage = Some(f64::from(&v));
                }
                #[cfg(not(feature = "emucs"))]
                OBIS::InstantaneousCurrent(l, UFixedInteger(a)) => {
                    state.lines[l as usize].current = Some(a);
                }
                #[cfg(feature = "emucs")]
                OBIS::InstantaneousCurrent(l, c) => {
                    state.lines[l as usize].current = Some(f64::from(&c));
                }
                OBIS::InstantaneousActivePowerPlus(l, p) => {
                    state.lines[l as usize].active_power_plus = Some(f64::from(&p));
                }
                OBIS::InstantaneousActivePowerNeg(l, p) => {
                    state.lines[l as usize].active_power_neg = Some(f64::from(&p));
                }
                OBIS::SlaveDeviceType(s, UFixedInteger(dt)) => {
                    state.slaves[s as usize].device_type = Some(dt);
                }
                OBIS::SlaveMeterReading(s, tst, mr) => {
                    state.slaves[s as usize].meter_reading = Some((tst, f64::from(&mr)));
                }
                #[cfg(feature = "emucs")]
                OBIS::SlaveValveState(s, UFixedInteger(v)) => {
                    state.slaves[s as usize].valve_state = Some(v);
                }
                _ => {} // Ignore rest.
            }
            Ok(state)
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn example() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/isk.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();
        let state = crate::Result::<crate::state::State>::from(&telegram).unwrap();

        assert_eq!(
            state.datetime.as_ref().unwrap(),
            &crate::types::TST {
                year: 19,
                month: 3,
                day: 20,
                hour: 18,
                minute: 14,
                second: 3,
                dst: false
            }
        );

        use crate::obis::Tariff::*;

        assert_eq!(state.meterreadings[Tariff1 as usize].to.unwrap(), 576.239);
        assert_eq!(state.meterreadings[Tariff2 as usize].to.unwrap(), 465.162);
        assert_eq!(state.tariff_indicator.unwrap(), [0, 2]);

        eprintln!("{:?}", state);
    }
}
