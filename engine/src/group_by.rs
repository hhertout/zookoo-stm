//! Grouping utilities for targets by scrape interval

use std::collections::HashMap;

use configuration::model::{HasScrapeInterval, ScrapeInterval};

/// Group targets by their scrape interval.
///
/// If a target's interval equals `M1` (the default), it uses the probe's default interval instead.
/// This allows targets to inherit the probe-level interval while still allowing per-target overrides.
///
/// # Example
/// ```ignore
/// let targets = vec![target1, target2, target3];
/// let grouped = group_by_interval(&targets, ScrapeInterval::S30);
/// // Returns HashMap<ScrapeInterval, Vec<Target>>
/// ```
pub fn group_by_interval<T: HasScrapeInterval + Clone>(
    targets: &[T],
    default_interval: ScrapeInterval,
) -> HashMap<ScrapeInterval, Vec<T>> {
    let mut groups: HashMap<ScrapeInterval, Vec<T>> = HashMap::new();

    for target in targets {
        let interval = target.scrape_interval();
        // Use target's interval if different from default (M1), otherwise use probe's default
        let effective_interval =
            if interval == ScrapeInterval::M1 { default_interval } else { interval };
        groups.entry(effective_interval).or_default().push(target.clone());
    }

    groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    struct MockTarget {
        interval: ScrapeInterval,
    }

    impl HasScrapeInterval for MockTarget {
        fn scrape_interval(&self) -> ScrapeInterval {
            self.interval
        }
    }

    #[test]
    fn test_group_by_single_interval() {
        let targets = vec![
            MockTarget { interval: ScrapeInterval::S10 },
            MockTarget { interval: ScrapeInterval::S10 },
        ];

        let grouped = group_by_interval(&targets, ScrapeInterval::S30);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped.get(&ScrapeInterval::S10).unwrap().len(), 2);
    }

    #[test]
    fn test_group_by_multiple_intervals() {
        let targets = vec![
            MockTarget { interval: ScrapeInterval::S10 },
            MockTarget { interval: ScrapeInterval::S30 },
            MockTarget { interval: ScrapeInterval::S10 },
        ];

        let grouped = group_by_interval(&targets, ScrapeInterval::M1);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&ScrapeInterval::S10).unwrap().len(), 2);
        assert_eq!(grouped.get(&ScrapeInterval::S30).unwrap().len(), 1);
    }

    #[test]
    fn test_group_by_uses_default_for_m1() {
        let targets = vec![
            MockTarget { interval: ScrapeInterval::M1 }, // Should use default
            MockTarget { interval: ScrapeInterval::S10 },
        ];

        let grouped = group_by_interval(&targets, ScrapeInterval::S30);

        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped.get(&ScrapeInterval::S30).unwrap().len(), 1); // M1 became S30
        assert_eq!(grouped.get(&ScrapeInterval::S10).unwrap().len(), 1);
    }

    #[test]
    fn test_group_by_all_use_default() {
        let targets = vec![
            MockTarget { interval: ScrapeInterval::M1 },
            MockTarget { interval: ScrapeInterval::M1 },
        ];

        let grouped = group_by_interval(&targets, ScrapeInterval::M5);

        assert_eq!(grouped.len(), 1);
        assert_eq!(grouped.get(&ScrapeInterval::M5).unwrap().len(), 2);
    }
}
