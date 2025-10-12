use crate::config::scrape_interval::ScrapeInterval;

/// GroupByInterval is a struct that holds vectors of items grouped by different time intervals.
/// It provides methods to access and manipulate these groups, allowing for efficient handling of data that is categorized by time intervals.
/// The intervals include 5 seconds, 10 seconds, 30 seconds, 1 minute, 5 minutes, 10 minutes, 30 minutes, 1 hour, 12 hours, 1 day, 7 days, and 30 days.
/// Each group is represented as a vector of items of type T, allowing for flexible storage of various data types.
///
#[derive(Debug, Clone)]
pub struct GroupByInterval<T> {
    pub s5: Vec<T>,
    pub s10: Vec<T>,
    pub s30: Vec<T>,
    pub m1: Vec<T>,
    pub m5: Vec<T>,
    pub m10: Vec<T>,
    pub m30: Vec<T>,
    pub h1: Vec<T>,
    pub h12: Vec<T>,
    pub d1: Vec<T>,
    pub d7: Vec<T>,
    pub d30: Vec<T>,
}

impl<T> IntoIterator for GroupByInterval<T> {
    type Item = (ScrapeInterval, Vec<T>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            (ScrapeInterval::S5, self.s5),
            (ScrapeInterval::S10, self.s10),
            (ScrapeInterval::S30, self.s30),
            (ScrapeInterval::M1, self.m1),
            (ScrapeInterval::M5, self.m5),
            (ScrapeInterval::M10, self.m10),
            (ScrapeInterval::M30, self.m30),
            (ScrapeInterval::H1, self.h1),
            (ScrapeInterval::H12, self.h12),
            (ScrapeInterval::D1, self.d1),
            (ScrapeInterval::D7, self.d7),
            (ScrapeInterval::D30, self.d30),
        ]
        .into_iter()
    }
}

impl<T: Clone> GroupByInterval<T> {
    pub fn new() -> Self {
        GroupByInterval {
            s5: vec![],
            s10: vec![],
            s30: vec![],
            m1: vec![],
            m5: vec![],
            m10: vec![],
            m30: vec![],
            h1: vec![],
            h12: vec![],
            d1: vec![],
            d7: vec![],
            d30: vec![],
        }
    }

    pub fn get_mut(&mut self, interval: &ScrapeInterval) -> &mut Vec<T> {
        match interval {
            ScrapeInterval::S5 => &mut self.s5,
            ScrapeInterval::S10 => &mut self.s10,
            ScrapeInterval::S30 => &mut self.s30,
            ScrapeInterval::M1 => &mut self.m1,
            ScrapeInterval::M5 => &mut self.m5,
            ScrapeInterval::M10 => &mut self.m10,
            ScrapeInterval::M30 => &mut self.m30,
            ScrapeInterval::H1 => &mut self.h1,
            ScrapeInterval::H12 => &mut self.h12,
            ScrapeInterval::D1 => &mut self.d1,
            ScrapeInterval::D7 => &mut self.d7,
            ScrapeInterval::D30 => &mut self.d30,
        }
    }

    pub fn get(&self, interval: &ScrapeInterval) -> &Vec<T> {
        match interval {
            ScrapeInterval::S5 => &self.s5,
            ScrapeInterval::S10 => &self.s10,
            ScrapeInterval::S30 => &self.s30,
            ScrapeInterval::M1 => &self.m1,
            ScrapeInterval::M5 => &self.m5,
            ScrapeInterval::M10 => &self.m10,
            ScrapeInterval::M30 => &self.m30,
            ScrapeInterval::H1 => &self.h1,
            ScrapeInterval::H12 => &self.h12,
            ScrapeInterval::D1 => &self.d1,
            ScrapeInterval::D7 => &self.d7,
            ScrapeInterval::D30 => &self.d30,
        }
    }

    pub fn iter(&self) -> Vec<(ScrapeInterval, &Vec<T>)> {
        vec![
            (ScrapeInterval::S5, &self.s5),
            (ScrapeInterval::S10, &self.s10),
            (ScrapeInterval::S30, &self.s30),
            (ScrapeInterval::M1, &self.m1),
            (ScrapeInterval::M5, &self.m5),
            (ScrapeInterval::M10, &self.m10),
            (ScrapeInterval::M30, &self.m30),
            (ScrapeInterval::H1, &self.h1),
            (ScrapeInterval::H12, &self.h12),
            (ScrapeInterval::D1, &self.d1),
            (ScrapeInterval::D7, &self.d7),
            (ScrapeInterval::D30, &self.d30),
        ]
    }

    /// Merge two GroupByInterval instances by concatenating their vectors for each interval.
    /// This method allows for combining the data from two instances, effectively merging their contents.
    /// It returns a new GroupByInterval instance that contains the combined data from both instances.
    pub fn merge(&self, other: GroupByInterval<T>) -> GroupByInterval<T> {
        GroupByInterval {
            s5: [self.s5.clone(), other.s5.clone()].concat(),
            s10: [self.s10.clone(), other.s10.clone()].concat(),
            s30: [self.s30.clone(), other.s30.clone()].concat(),
            m1: [self.m1.clone(), other.m1.clone()].concat(),
            m5: [self.m5.clone(), other.m5.clone()].concat(),
            m10: [self.m10.clone(), other.m10.clone()].concat(),
            m30: [self.m30.clone(), other.m30.clone()].concat(),
            h1: [self.h1.clone(), other.h1.clone()].concat(),
            h12: [self.h12.clone(), other.h12.clone()].concat(),
            d1: [self.d1.clone(), other.d1.clone()].concat(),
            d7: [self.d7.clone(), other.d7.clone()].concat(),
            d30: [self.d30.clone(), other.d30.clone()].concat(),
        }
    }
}
