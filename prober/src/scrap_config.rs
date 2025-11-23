use crate::{
    config::{
        self,
        scrape_interval::ScrapeInterval,
        target::{HttpTarget, IcmpTarget},
    },
    group_by_interval::GroupByInterval,
};

#[derive(Debug, Clone)]
pub struct ProbeConfig {
    pub scrap_config: config::ScrapConfiguration,
}

impl ProbeConfig {
    pub fn new(scrap_config: config::ScrapConfiguration) -> Self {
        return ProbeConfig { scrap_config };
    }

    /// Apply the labels coming from the default object of the configuration
    pub fn apply_default_labels(&mut self) -> &mut Self {
        let default_labels = self.scrap_config.default.to_labels_hashmap();

        if let Some(http_target) = self.scrap_config.http.as_mut() {
            for target in http_target.targets.iter_mut() {
                if let Some(labels_arc) = &target.labels {
                    // Create a new HashMap by merging existing and default labels
                    let mut merged_labels = (**labels_arc).clone();
                    for (k, v) in default_labels.iter() {
                        merged_labels.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                    target.labels = Some(std::sync::Arc::new(merged_labels));
                } else {
                    // If no labels, just set default labels
                    target.labels = Some(std::sync::Arc::new(default_labels.clone()));
                }
            }
        }

        if let Some(icmp_target) = self.scrap_config.icmp.as_mut() {
            for target in icmp_target.targets.iter_mut() {
                if let Some(labels_arc) = &target.labels {
                    // Create a new HashMap by merging existing and default labels
                    let mut merged_labels = (**labels_arc).clone();
                    for (k, v) in default_labels.iter() {
                        merged_labels.entry(k.clone()).or_insert_with(|| v.clone());
                    }
                    target.labels = Some(std::sync::Arc::new(merged_labels));
                } else {
                    // If no labels, just set default labels
                    target.labels = Some(std::sync::Arc::new(default_labels.clone()));
                }
            }
        }

        return self;
    }

    /// Get the group of ICMP targets grouped by their scrape intervals.
    /// This method iterates over the ICMP targets defined in the configuration and groups them based on their scrape intervals.
    /// It returns a `GroupByInterval<IcmpTarget>` instance containing the targets categorized by their respective intervals.
    pub fn icmp_group_by_interval(&self) -> GroupByInterval<IcmpTarget> {
        let mut group_by: GroupByInterval<IcmpTarget> = GroupByInterval::new();
        let _ = self.scrap_config.icmp.as_ref().map(|icmp_target| {
            for target in icmp_target.targets.iter() {
                match target.scrape_interval {
                    ScrapeInterval::S5 => group_by.s5.push(target.clone()),
                    ScrapeInterval::S10 => group_by.s10.push(target.clone()),
                    ScrapeInterval::S30 => group_by.s30.push(target.clone()),
                    ScrapeInterval::M1 => group_by.m1.push(target.clone()),
                    ScrapeInterval::M5 => group_by.m5.push(target.clone()),
                    ScrapeInterval::M10 => group_by.m10.push(target.clone()),
                    ScrapeInterval::M30 => group_by.m30.push(target.clone()),
                    ScrapeInterval::H1 => group_by.h1.push(target.clone()),
                    ScrapeInterval::H12 => group_by.h12.push(target.clone()),
                    ScrapeInterval::D1 => group_by.d1.push(target.clone()),
                    ScrapeInterval::D7 => group_by.d7.push(target.clone()),
                    ScrapeInterval::D30 => group_by.d30.push(target.clone()),
                }
            }
        });

        return group_by;
    }

    /// Get the group of HTTP targets grouped by their scrape intervals.
    /// This method iterates over the HTTP targets defined in the configuration and groups them based on their scrape intervals.
    /// It returns a `GroupByInterval<HttpTarget>` instance containing the targets categorized by their respective intervals.
    pub fn http_group_by_interval(&self) -> GroupByInterval<HttpTarget> {
        let mut group_by: GroupByInterval<HttpTarget> = GroupByInterval::new();
        let _ = self.scrap_config.http.as_ref().map(|http_target| {
            for target in http_target.targets.iter() {
                match target.scrape_interval {
                    ScrapeInterval::S5 => group_by.s5.push(target.clone()),
                    ScrapeInterval::S10 => group_by.s10.push(target.clone()),
                    ScrapeInterval::S30 => group_by.s30.push(target.clone()),
                    ScrapeInterval::M1 => group_by.m1.push(target.clone()),
                    ScrapeInterval::M5 => group_by.m5.push(target.clone()),
                    ScrapeInterval::M10 => group_by.m10.push(target.clone()),
                    ScrapeInterval::M30 => group_by.m30.push(target.clone()),
                    ScrapeInterval::H1 => group_by.h1.push(target.clone()),
                    ScrapeInterval::H12 => group_by.h12.push(target.clone()),
                    ScrapeInterval::D1 => group_by.d1.push(target.clone()),
                    ScrapeInterval::D7 => group_by.d7.push(target.clone()),
                    ScrapeInterval::D30 => group_by.d30.push(target.clone()),
                }
            }
        });

        return group_by;
    }
}

impl From<configuration::model::Configuration> for ProbeConfig {
    fn from(value: configuration::model::Configuration) -> Self {
        ProbeConfig {
            scrap_config: config::ScrapConfiguration::from(value),
        }
    }
}
