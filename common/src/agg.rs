use std::ops::Div;
use crate::{DataPoint, Int};

/// The Manager for handling multiple datapoints
pub struct Aggregator;

impl Aggregator {
    pub fn agg(datapoints: &[DataPoint]) -> DataPoint {
        let value = Int::from(0);
        let timestamp = 0u32;
        for d in datapoints {
            value.checked_add(d.value).expect("value overflow");
            timestamp.checked_add(d.timestamp).expect("timestamp overflow");
        }
        let l = datapoints.len();
        DataPoint::new(value.div(l), timestamp / l as u32)
    }
}
