use crate::climate_data::ClimateData;
use async_trait::async_trait;
use std::cmp::Ordering;
use std::error::Error;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Trend {
    Up,
    Down,
    None,
}

#[async_trait]
pub trait DataReaction<T: PartialOrd> {
    const PERIOD: Duration;
    const TREND: Trend;

    fn get_value(data: &ClimateData) -> T;
    fn only_if(latest_data: &ClimateData) -> bool;

    #[allow(unused)]
    fn force_run(latest_data: &ClimateData) -> bool {
        false
    }

    async fn run() -> Result<(), Box<dyn Error>>;

    fn validate(values: &[ClimateData]) -> bool {
        let self_type_name = std::any::type_name::<Self>();

        let last_data = if let Some(last_data) = values.last() {
            last_data
        } else {
            tracing::debug!(?self_type_name, "reaction not needed – no data");
            return false;
        };

        if Self::force_run(last_data) {
            tracing::debug!(?self_type_name, "reaction forced");
            return true;
        }

        let only_if = Self::only_if(last_data);
        if !only_if {
            tracing::debug!(
                ?self_type_name,
                "reaction not needed – only_if returned false"
            );
            return false;
        }

        // we only run reaction once per specified period of time
        let period_size = Self::PERIOD.as_secs() / 5;
        if values.len() < period_size as usize {
            tracing::debug!(?self_type_name, "reaction not needed – not enough data");
            return false;
        }

        if values.len() % period_size as usize != 0 {
            tracing::debug!(
                ?self_type_name,
                "reaction not needed – waiting for next period"
            );
            return false;
        }

        let trend_check_values = &values[values.len() - period_size as usize..];
        let current_trend = Self::get_trend(trend_check_values);
        let trend_sync = current_trend == Self::TREND;

        if !trend_sync {
            tracing::debug!(
                ?self_type_name,
                "reaction not needed – trend is not matching. Expected {:?}, got {current_trend:?}",
                Self::TREND
            );
            return false;
        }

        trend_sync
    }

    fn get_trend(values: &[ClimateData]) -> Trend {
        let len = values.len();
        if len < 2 {
            return Trend::None;
        }

        let mut up_count = 0;
        let mut down_count = 0;

        for i in 1..len {
            let a = Self::get_value(&values[i - 1]);
            let b = Self::get_value(&values[i]);

            if a > b {
                up_count += 1;
            } else if a < b {
                down_count += 1;
            }
        }

        match up_count.cmp(&down_count) {
            Ordering::Greater => Trend::Up,
            Ordering::Less => Trend::Down,
            Ordering::Equal => Trend::None,
        }
    }
}
