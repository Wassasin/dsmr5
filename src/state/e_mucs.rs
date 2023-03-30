use serde::{Deserialize, Serialize};

use crate::{obis::e_mucs::*, state::common, types::*, Result};
use core::ops::Deref;

/// One of three possible lines in the meter.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Line {
    pub parent: common::Line,
    pub current: Option<f64>,
}

impl Deref for Line {
    type Target = common::Line;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

/// One of 4 possible slaves to the meter.
///
/// Such as a gas meter, water meter or heat supply.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct Slave {
    pub parent: common::Slave,
    pub valve_state: Option<u64>,
}

impl Deref for Slave {
    type Target = common::Slave;

    fn deref(&self) -> &Self::Target {
        &self.parent
    }
}

/// The metering state surmised for a single Telegram.
#[derive(Default, Debug, Serialize, Deserialize)]
pub struct State {
    pub parent: common::State,
    pub breaker_state: Option<u64>,
    pub limiter_threshold: Option<f64>,
    pub fuse_supervision_threshold: Option<u64>,
    pub average_demand: Option<f64>,
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
            DSMR4(o) => self.parent.apply(o)?,
            InstantaneousCurrent(l, c) => {
                self.lines[l as usize].current = Some(f64::from(&c));
            }
            SlaveMeterReadingNonCorrected(_, _, _) => {}
            SlaveValveState(s, UFixedInteger(v)) => {
                self.slaves[s as usize].valve_state = Some(v);
            }
            BreakerState(UFixedInteger(b)) => {
                self.breaker_state = Some(b);
            }
            LimiterThreshold(l) => {
                self.limiter_threshold = Some(f64::from(&l));
            }
            FuseSupervisionThreshold(UFixedInteger(f)) => {
                self.fuse_supervision_threshold = Some(f);
            }
            CurrentAverageDemand(f) => {
                self.average_demand = Some(f64::from(&f));
            }
            MaximumDemandMonth(_, _) => {}
            MaximumDemandYear => {}
        };
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
    fn example_flu() {
        let mut buffer = [0u8; 2048];
        let file = std::fs::read("test/flu.txt").unwrap();

        let (left, _right) = buffer.split_at_mut(file.len());
        left.copy_from_slice(file.as_slice());

        let readout = crate::Readout { buffer };
        let telegram = readout.to_telegram().unwrap();
        let state = crate::Result::<crate::state::e_mucs::State>::from(&telegram).unwrap();

        assert_eq!(
            state.datetime.as_ref().unwrap(),
            &crate::types::TST {
                year: 23,
                month: 2,
                day: 11,
                hour: 11,
                minute: 15,
                second: 41,
                dst: false
            }
        );

        use crate::obis::Tariff::*;

        assert_eq!(state.meterreadings[Tariff1 as usize].to.unwrap(), 1114.057);
        assert_eq!(state.meterreadings[Tariff1 as usize].by.unwrap(), 0.407);
        assert_eq!(state.meterreadings[Tariff2 as usize].to.unwrap(), 997.282);
        assert_eq!(state.meterreadings[Tariff2 as usize].by.unwrap(), 0.281);
        assert_eq!(state.tariff_indicator.unwrap(), [0, 2]);
        assert_eq!(state.power_delivered.unwrap(), 0.031);
        assert_eq!(state.power_received.unwrap(), 0.0);
        assert_eq!(state.breaker_state.unwrap(), 1);
        assert_eq!(state.limiter_threshold.unwrap(), 999.9);
        assert_eq!(state.fuse_supervision_threshold.unwrap(), 999);
        assert_eq!(state.average_demand.unwrap(), 0.0);
        assert_eq!(state.slaves[0].valve_state.unwrap(), 1);

        eprintln!("{:?}", state);
    }
}
