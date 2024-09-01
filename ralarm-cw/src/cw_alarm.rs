use ralarm::{utils::EvictingQue, Alarm, AlarmState};

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TreatMissingData {
    Breaching,
    NotBreaching,
    Ignore,
    Missing,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ComparisonOperator {
    G,
    GEq,
    L,
    LEq,
}

#[derive(Debug, PartialEq, Copy, Clone)]
enum DpState {
    Good,
    Bad,
    Missing,
}

pub struct CwAlarm {
    evaluation_periods: usize,
    dps_to_alarm: usize,

    threshold: f64,
    treat_missing_data: TreatMissingData,
    comparison_operator: ComparisonOperator,

    lookback: EvictingQue<DpState>,

    state: AlarmState,
}

impl CwAlarm {
    fn get_dp_state(&self, dp: Option<f64>) -> DpState {
        match dp {
            Some(dp) => {
                let bad = match self.comparison_operator {
                    ComparisonOperator::G => dp > self.threshold,
                    ComparisonOperator::GEq => dp >= self.threshold,
                    ComparisonOperator::L => dp < self.threshold,
                    ComparisonOperator::LEq => dp <= self.threshold,
                };

                match bad {
                    true => DpState::Bad,
                    false => DpState::Good,
                }
            }
            None => DpState::Missing,
        }
    }
}

impl Alarm<Option<f64>> for CwAlarm {
    fn feed(&mut self, val: Option<f64>) -> AlarmState {
        let new_dp_state = self.get_dp_state(val);

        if new_dp_state == DpState::Missing && self.treat_missing_data == TreatMissingData::Ignore {
            return self.state;
        };

        self.lookback.push(new_dp_state);

        let mut bad_dp_count = 0;
        for &dp_state in &self.lookback {
            match dp_state {
                DpState::Good => (),
                DpState::Bad => bad_dp_count += 1,
                DpState::Missing => match self.treat_missing_data {
                    TreatMissingData::Breaching => bad_dp_count += 1,
                    _ => (),
                },
            }
        }

        if bad_dp_count >= self.dps_to_alarm {
            self.state = AlarmState::Alarm;
        } else {
            self.state = AlarmState::Ok;
        }

        self.state
    }
}

impl CwAlarm {
    pub fn evaluation_periods(&self) -> usize {
        self.evaluation_periods
    }

    pub fn dps_to_alarm(&self) -> usize {
        self.dps_to_alarm
    }

    pub fn threshold(&self) -> f64 {
        self.threshold
    }

    pub fn treat_missing_data(&self) -> TreatMissingData {
        self.treat_missing_data
    }

    pub fn comparison_operator(&self) -> ComparisonOperator {
        self.comparison_operator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_stay_ok() {
        let mut alarm = CwAlarmBuilder::default()
            .threshold(10.0)
            .comparison_operator(ComparisonOperator::G)
            .evaluation_periods(5)
            .treat_missing_data(TreatMissingData::Breaching)
            .dps_to_alarm(5)
            .build();

        assert_eq!(alarm.feed(Some(9.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(9.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(9.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(9.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(9.0)), AlarmState::Ok);
    }

    #[test]
    fn should_transition_to_alarm() {
        let mut alarm = CwAlarmBuilder::default()
            .threshold(10.0)
            .comparison_operator(ComparisonOperator::G)
            .evaluation_periods(5)
            .treat_missing_data(TreatMissingData::Breaching)
            .dps_to_alarm(5)
            .build();

        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Alarm);
    }

    #[test]
    fn should_transition_back_to_ok() {
        let mut alarm = CwAlarmBuilder::default()
            .threshold(10.0)
            .comparison_operator(ComparisonOperator::G)
            .evaluation_periods(5)
            .treat_missing_data(TreatMissingData::Breaching)
            .dps_to_alarm(5)
            .build();

        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Alarm);
        assert_eq!(alarm.feed(Some(9.0)), AlarmState::Ok);
    }

    #[test]
    fn should_handle_missing_dps_breaching() {
        let mut alarm = CwAlarmBuilder::default()
            .threshold(10.0)
            .comparison_operator(ComparisonOperator::G)
            .evaluation_periods(5)
            .treat_missing_data(TreatMissingData::Breaching)
            .dps_to_alarm(5)
            .build();

        assert_eq!(alarm.feed(None), AlarmState::Ok);
        assert_eq!(alarm.feed(None), AlarmState::Ok);
        assert_eq!(alarm.feed(None), AlarmState::Ok);
        assert_eq!(alarm.feed(None), AlarmState::Ok);
        assert_eq!(alarm.feed(None), AlarmState::Alarm);
    }

    #[test]
    fn should_handle_missing_dps_maintain() {
        let mut alarm = CwAlarmBuilder::default()
            .threshold(10.0)
            .comparison_operator(ComparisonOperator::G)
            .evaluation_periods(5)
            .treat_missing_data(TreatMissingData::Ignore)
            .dps_to_alarm(5)
            .build();

        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Ok);
        assert_eq!(alarm.feed(Some(11.0)), AlarmState::Alarm);
        assert_eq!(alarm.feed(None), AlarmState::Alarm);
        assert_eq!(alarm.feed(None), AlarmState::Alarm);
        assert_eq!(alarm.feed(None), AlarmState::Alarm);
        assert_eq!(alarm.feed(None), AlarmState::Alarm);
        assert_eq!(alarm.feed(None), AlarmState::Alarm);
    }
}

pub use builder::CwAlarmBuilder;

mod builder {
    use super::*;
    use std::marker::PhantomData;

    pub enum Set {}
    pub enum Unset {}

    pub struct CwAlarmBuilder<E, T, C> {
        evaluation_periods: Option<usize>,
        dps_to_alarm: Option<usize>,
        threshold: Option<f64>,
        treat_missing_data: Option<TreatMissingData>,
        comparison_operator: Option<ComparisonOperator>,

        type_state: PhantomData<(E, T, C)>,
    }

    impl Default for CwAlarmBuilder<Unset, Unset, Unset> {
        fn default() -> Self {
            Self {
                evaluation_periods: None,
                dps_to_alarm: None,
                threshold: None,
                treat_missing_data: None,
                comparison_operator: None,

                type_state: PhantomData,
            }
        }
    }

    impl<E, T, C> CwAlarmBuilder<E, T, C> {
        pub fn evaluation_periods(self, val: usize) -> CwAlarmBuilder<Set, T, C> {
            CwAlarmBuilder {
                evaluation_periods: Some(val),
                dps_to_alarm: self.dps_to_alarm,
                threshold: self.threshold,
                treat_missing_data: self.treat_missing_data,
                comparison_operator: self.comparison_operator,

                type_state: PhantomData,
            }
        }

        pub fn dps_to_alarm(self, val: usize) -> CwAlarmBuilder<E, T, C> {
            CwAlarmBuilder {
                evaluation_periods: self.evaluation_periods,
                dps_to_alarm: Some(val),
                threshold: self.threshold,
                treat_missing_data: self.treat_missing_data,
                comparison_operator: self.comparison_operator,

                type_state: PhantomData,
            }
        }

        pub fn threshold(self, val: f64) -> CwAlarmBuilder<E, Set, C> {
            CwAlarmBuilder {
                evaluation_periods: self.evaluation_periods,
                dps_to_alarm: self.dps_to_alarm,
                threshold: Some(val),
                treat_missing_data: self.treat_missing_data,
                comparison_operator: self.comparison_operator,

                type_state: PhantomData,
            }
        }

        pub fn treat_missing_data(self, val: TreatMissingData) -> CwAlarmBuilder<E, T, C> {
            CwAlarmBuilder {
                evaluation_periods: self.evaluation_periods,
                dps_to_alarm: self.dps_to_alarm,
                threshold: self.threshold,
                treat_missing_data: Some(val),
                comparison_operator: self.comparison_operator,

                type_state: PhantomData,
            }
        }

        pub fn comparison_operator(self, val: ComparisonOperator) -> CwAlarmBuilder<E, T, Set> {
            CwAlarmBuilder {
                evaluation_periods: self.evaluation_periods,
                dps_to_alarm: self.dps_to_alarm,
                threshold: self.threshold,
                treat_missing_data: self.treat_missing_data,
                comparison_operator: Some(val),

                type_state: PhantomData,
            }
        }
    }

    impl CwAlarmBuilder<Set, Set, Set> {
        pub fn build(self) -> CwAlarm {
            let evaluation_periods = self.evaluation_periods.unwrap_or_else(|| unreachable!());
            let dps_to_alarm = self.dps_to_alarm.unwrap_or(evaluation_periods);

            let threshold = self.threshold.unwrap_or_else(|| unreachable!());
            let treat_missing_data = self.treat_missing_data.unwrap_or(TreatMissingData::Missing);
            let comparison_operator = self.comparison_operator.unwrap_or_else(|| unreachable!());

            CwAlarm {
                evaluation_periods,
                dps_to_alarm,

                threshold,
                treat_missing_data,
                comparison_operator,

                lookback: EvictingQue::new(evaluation_periods),

                state: AlarmState::Ok,
            }
        }
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn should_set_values() {
            let evaluation_periods: usize = 5;
            let dps_to_alarm: usize = 2;
            let threshold = 5.0;
            let treat_missing_data = TreatMissingData::Breaching;
            let comparison_operator = ComparisonOperator::G;

            let alarm = CwAlarmBuilder::default()
                .evaluation_periods(evaluation_periods)
                .dps_to_alarm(dps_to_alarm)
                .threshold(threshold)
                .treat_missing_data(treat_missing_data)
                .comparison_operator(comparison_operator)
                .build();

            assert_eq!(alarm.evaluation_periods, evaluation_periods);
            assert_eq!(alarm.dps_to_alarm, dps_to_alarm);
            assert_eq!(alarm.threshold, threshold);
            assert_eq!(alarm.treat_missing_data, treat_missing_data);
            assert_eq!(alarm.comparison_operator, comparison_operator);
        }
    }
}
