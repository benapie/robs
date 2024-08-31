use ralarm::{Alarm, AlarmState};

#[derive(PartialEq)]
pub enum CwAlarm {
    A,
    B,
}

impl Alarm<f64> for CwAlarm {
    fn feed(&mut self, _: f64) -> AlarmState {
        if *self == CwAlarm::A {
            return AlarmState::Alarm;
        }
        AlarmState::Alarm
    }
}

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
