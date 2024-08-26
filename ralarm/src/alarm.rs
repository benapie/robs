#[derive(Copy, Clone, Debug, PartialEq)]
pub enum AlarmState {
    Alarm,
    Ok,
}

pub trait Alarm<T> {
    fn feed(&mut self, val: T) -> AlarmState;
}
