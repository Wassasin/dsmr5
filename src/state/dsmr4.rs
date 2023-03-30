//! Convenience structs to get and keep the current state of the meter in memory.
//!
//! Usage of these types is entirely optional.
//! When only needing a few or single record, it is more efficient to directly filter on the
//! telegram objects iterator.

use core::ops::Deref;
use serde::{Deserialize, Serialize};

use crate::{obis::dsmr4::*, state::common, types::*, Result};

pub use common::{Line, MeterReading, Slave};

/// The metering state surmised for a single Telegram.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct State {
    pub parent: common::State,
    pub lines: [Line; 3],
    pub slaves: [Slave; 4],
}

impl Deref for State {
    type Target = common::State;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl State {
    pub fn apply(&mut self, o: OBIS) -> Result<()> {
        use OBIS::*;
        match o {
            VoltageSags(l, UFixedInteger(n)) => {
                self.lines[l as usize].voltage_sags = Some(n);
            }
            VoltageSwells(l, UFixedInteger(n)) => {
                self.lines[l as usize].voltage_swells = Some(n);
            }
            InstantaneousVoltage(l, v) => {
                self.lines[l as usize].voltage = Some(f64::from(&v));
            }
            InstantaneousActivePowerPlus(l, p) => {
                self.lines[l as usize].active_power_plus = Some(f64::from(&p));
            }
            InstantaneousActivePowerNeg(l, p) => {
                self.lines[l as usize].active_power_neg = Some(f64::from(&p));
            }
            InstantaneousCurrent(l, UFixedInteger(a)) => {
                self.lines[l as usize].current = Some(a);
            }
            SlaveDeviceType(s, UFixedInteger(dt)) => {
                self.slaves[s as usize].device_type = Some(dt);
            }
            SlaveMeterReading(s, tst, mr) => {
                self.slaves[s as usize].meter_reading = Some((tst, f64::from(&mr)));
            }
            o => {
                self.parent.apply(o)?;
            }
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
        let state = crate::Result::<crate::state::dsmr4::State>::from(&telegram).unwrap();

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
        assert_eq!(state.power_delivered.unwrap(), 0.193);
        assert_eq!(state.power_failures.unwrap(), 9);
        assert_eq!(state.long_power_failures.unwrap(), 8);
        assert_eq!(
            state.lines[0],
            crate::state::dsmr4::Line {
                voltage_sags: Some(6,),
                voltage_swells: Some(1,),
                voltage: Some(236.1,),
                active_power_plus: Some(0.193,),
                active_power_neg: Some(0.0,),
                current: Some(1,),
            }
        );
        assert_eq!(
            state.slaves[0],
            crate::state::dsmr4::Slave {
                device_type: Some(3,),
                meter_reading: Some((
                    crate::types::TST {
                        year: 19,
                        month: 3,
                        day: 20,
                        hour: 18,
                        minute: 10,
                        second: 3,
                        dst: false,
                    },
                    304.089,
                ),),
            }
        );

        eprintln!("{:?}", state);
    }
}
