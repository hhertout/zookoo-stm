use std::process::exit;

use crate::{
    config::{self, scrap_interval::ScrapInterval, target::HttpTarget},
    file::json_parser,
    group_by_interval::GroupByInterval,
};

#[derive(Debug, Clone)]
pub struct ProbeConfig {
    pub config: config::ScrapConfiguration,
}

impl ProbeConfig {
    pub fn new(config: config::ScrapConfiguration) -> Self {
        return ProbeConfig { config };
    }

    pub fn http_group_by_interval(&self) -> GroupByInterval<HttpTarget> {
        let mut group_by: GroupByInterval<HttpTarget> = GroupByInterval::new();
        let _ = self.config.http.as_ref().map(|http_target| {
            if let Some(targets) = http_target.targets.clone() {
                for target in targets {
                    match target.scrap_interval {
                        ScrapInterval::S5 => group_by.s5.push(target),
                        ScrapInterval::S10 => group_by.s10.push(target),
                        ScrapInterval::S30 => group_by.s30.push(target),
                        ScrapInterval::M1 => group_by.m1.push(target),
                        ScrapInterval::M5 => group_by.m5.push(target),
                        ScrapInterval::M10 => group_by.m10.push(target),
                        ScrapInterval::M30 => group_by.m30.push(target),
                        ScrapInterval::H1 => group_by.h1.push(target),
                        ScrapInterval::H12 => group_by.h12.push(target),
                        ScrapInterval::D1 => group_by.d1.push(target),
                        ScrapInterval::D7 => group_by.d7.push(target),
                        ScrapInterval::D30 => group_by.d30.push(target),
                    }
                }
            }
        });

        return group_by;
    }

    pub fn json_http_group_by_interval(&self) -> GroupByInterval<HttpTarget> {
        let mut group_by: GroupByInterval<HttpTarget> = GroupByInterval::new();
        let _ = self.config.http.as_ref().map(|http_target| {
            if let Some(paths) = http_target.target_file.clone() {
                for path in paths {
                    let targets = match json_parser::parse_json_from_file(path) {
                        Ok(content) => content,
                        Err(err) => {
                            log::error!("{:?}", err);
                            exit(1)
                        }
                    };

                    for target in targets {
                        match target.scrap_interval {
                            ScrapInterval::S5 => group_by.s5.push(target),
                            ScrapInterval::S10 => group_by.s10.push(target),
                            ScrapInterval::S30 => group_by.s30.push(target),
                            ScrapInterval::M1 => group_by.m1.push(target),
                            ScrapInterval::M5 => group_by.m5.push(target),
                            ScrapInterval::M10 => group_by.m10.push(target),
                            ScrapInterval::M30 => group_by.m30.push(target),
                            ScrapInterval::H1 => group_by.h1.push(target),
                            ScrapInterval::H12 => group_by.h12.push(target),
                            ScrapInterval::D1 => group_by.d1.push(target),
                            ScrapInterval::D7 => group_by.d7.push(target),
                            ScrapInterval::D30 => group_by.d30.push(target),
                        }
                    }
                }
            }
        });

        return group_by;
    }
}

impl From<configuration::model::Configuration> for ProbeConfig {
    fn from(value: configuration::model::Configuration) -> Self {
        ProbeConfig {
            config: config::ScrapConfiguration::from(value),
        }
    }
}
