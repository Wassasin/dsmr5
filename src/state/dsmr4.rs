use core::ops::Deref;
use serde::{Deserialize, Serialize};

use crate::{
    obis::{dsmr4::*, Message},
    state::common,
    types::*,
    Error, Result,
};

pub use common::MeterReading;

/// One of three possible lines in the meter.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub voltage_sags: Option<u64>,
    pub voltage_swells: Option<u64>,
    pub active_power_plus: Option<f64>,
    pub active_power_neg: Option<f64>,
    pub current: Option<u64>,
}

impl Line {
    pub fn apply(&mut self, o: OBIS) -> Result<()> {
        use OBIS::*;
        match o {
            VoltageSags(_, UFixedInteger(n)) => {
                self.voltage_sags = Some(n);
            }
            VoltageSwells(_, UFixedInteger(n)) => {
                self.voltage_swells = Some(n);
            }
            InstantaneousActivePowerPlus(_, p) => {
                self.active_power_plus = Some(f64::from(&p));
            }
            InstantaneousActivePowerNeg(_, p) => {
                self.active_power_neg = Some(f64::from(&p));
            }
            InstantaneousCurrent(_, UFixedInteger(a)) => {
                self.current = Some(a);
            }
            _ => return Err(Error::ObisForgotten),
        }
        Ok(())
    }
}

/// One of 4 possible slaves to the meter.
///
/// Such as a gas meter, water meter or heat supply.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Slave {
    pub device_type: Option<u64>,
    pub meter_reading: Option<(TST, f64)>,
}

impl Slave {
    pub fn apply(&mut self, o: OBIS) -> Result<()> {
        use OBIS::*;
        match o {
            SlaveDeviceType(_, UFixedInteger(dt)) => {
                self.device_type = Some(dt);
            }
            SlaveMeterReading(_, tst, mr) => {
                self.meter_reading = Some((tst, f64::from(&mr)));
            }
            SlaveEquipmentIdentifier(_, _) => {}
            _ => return Err(Error::ObisForgotten),
        }
        Ok(())
    }
}

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
        if let Some(l) = o.line() {
            self.lines[l as usize].apply(o)
        } else if let Some(s) = o.slave() {
            self.slaves[s as usize].apply(o)
        } else {
            self.parent.apply(o)
        }
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
