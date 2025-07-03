use crate::config::scrap_interval::ScrapInterval;

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
    type Item = (ScrapInterval, Vec<T>);
    type IntoIter = std::vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        vec![
            (ScrapInterval::S5, self.s5),
            (ScrapInterval::S10, self.s10),
            (ScrapInterval::S30, self.s30),
            (ScrapInterval::M1, self.m1),
            (ScrapInterval::M5, self.m5),
            (ScrapInterval::M10, self.m10),
            (ScrapInterval::M30, self.m30),
            (ScrapInterval::H1, self.h1),
            (ScrapInterval::H12, self.h12),
            (ScrapInterval::D1, self.d1),
            (ScrapInterval::D7, self.d7),
            (ScrapInterval::D30, self.d30),
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

    pub fn get_mut(&mut self, interval: &ScrapInterval) -> &mut Vec<T> {
        match interval {
            ScrapInterval::S5 => &mut self.s5,
            ScrapInterval::S10 => &mut self.s10,
            ScrapInterval::S30 => &mut self.s30,
            ScrapInterval::M1 => &mut self.m1,
            ScrapInterval::M5 => &mut self.m5,
            ScrapInterval::M10 => &mut self.m10,
            ScrapInterval::M30 => &mut self.m30,
            ScrapInterval::H1 => &mut self.h1,
            ScrapInterval::H12 => &mut self.h12,
            ScrapInterval::D1 => &mut self.d1,
            ScrapInterval::D7 => &mut self.d7,
            ScrapInterval::D30 => &mut self.d30,
        }
    }

    pub fn get(&self, interval: &ScrapInterval) -> &Vec<T> {
        match interval {
            ScrapInterval::S5 => &self.s5,
            ScrapInterval::S10 => &self.s10,
            ScrapInterval::S30 => &self.s30,
            ScrapInterval::M1 => &self.m1,
            ScrapInterval::M5 => &self.m5,
            ScrapInterval::M10 => &self.m10,
            ScrapInterval::M30 => &self.m30,
            ScrapInterval::H1 => &self.h1,
            ScrapInterval::H12 => &self.h12,
            ScrapInterval::D1 => &self.d1,
            ScrapInterval::D7 => &self.d7,
            ScrapInterval::D30 => &self.d30,
        }
    }

    pub fn iter(&self) -> Vec<(ScrapInterval, &Vec<T>)> {
        vec![
            (ScrapInterval::S5, &self.s5),
            (ScrapInterval::S10, &self.s10),
            (ScrapInterval::S30, &self.s30),
            (ScrapInterval::M1, &self.m1),
            (ScrapInterval::M5, &self.m5),
            (ScrapInterval::M10, &self.m10),
            (ScrapInterval::M30, &self.m30),
            (ScrapInterval::H1, &self.h1),
            (ScrapInterval::H12, &self.h12),
            (ScrapInterval::D1, &self.d1),
            (ScrapInterval::D7, &self.d7),
            (ScrapInterval::D30, &self.d30),
        ]
    }

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
