use std::{collections::HashMap, time::Duration};

pub struct IcmpRequestMetrics {
    pub up: u8,
    pub duration: Duration,
    pub labels: Option<HashMap<String, String>>,
}
