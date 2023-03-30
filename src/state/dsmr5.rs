use serde::{Deserialize, Serialize};

use crate::{
    obis::{dsmr5::*, Message},
    state::{common, dsmr4},
    Error, Result,
};
use core::ops::Deref;

/// One of three possible lines in the meter.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Line {
    pub parent: dsmr4::Line,
    pub voltage: Option<f64>,
}

impl Deref for Line {
    type Target = dsmr4::Line;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

impl Line {
    pub fn apply(&mut self, o: OBIS) -> Result<()> {
        use OBIS::*;
        match o {
            InstantaneousVoltage(_, v) => {
                self.voltage = Some(f64::from(&v));
            }
            DSMR4(o) => self.parent.apply(o)?,
        }
        Ok(())
    }
}

#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
pub struct Slave(dsmr4::Slave);

impl From<dsmr4::Slave> for Slave {
    fn from(value: dsmr4::Slave) -> Self {
        Self(value)
    }
}

impl Deref for Slave {
    type Target = dsmr4::Slave;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Slave {
    pub fn apply(&mut self, o: OBIS) -> Result<()> {
        use OBIS::*;
        match o {
            DSMR4(o) => self.0.apply(o)?,
            _ => return Err(Error::ObisForgotten),
        }
        Ok(())
    }
}

/// The metering state surmised for a single Telegram.
#[derive(Default, Debug, Serialize, Deserialize, PartialEq)]
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
            use OBIS::*;
            match o {
                DSMR4(o) => self.parent.apply(o),
                _ => Err(Error::ObisForgotten),
            }
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

#[cfg(test)]
mod tests {
    #[test]
    fn example_isk() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/isk.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();

        let state = crate::Result::<crate::state::dsmr5::State>::from(&telegram).unwrap();

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
            crate::state::dsmr5::Line {
                parent: crate::state::dsmr4::Line {
                    voltage_sags: Some(6,),
                    voltage_swells: Some(1,),
                    active_power_plus: Some(0.193,),
                    active_power_neg: Some(0.0,),
                    current: Some(1,),
                },
                voltage: Some(236.1,),
            }
        );
        assert_eq!(
            state.slaves[0],
            crate::state::dsmr4::Slave {
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
                device_type: Some(3,),
            }
            .into()
        );

        eprintln!("{:?}", state);
    }
}
