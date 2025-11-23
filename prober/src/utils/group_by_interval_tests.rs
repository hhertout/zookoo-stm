#[cfg(test)]
mod tests {
    use crate::config::scrape_interval::ScrapeInterval;
    use crate::utils::GroupByInterval;

    #[test]
    fn test_new_creates_empty_groups() {
        let group: GroupByInterval<String> = GroupByInterval::new();
        
        assert!(group.s5.is_empty());
        assert!(group.s10.is_empty());
        assert!(group.s30.is_empty());
        assert!(group.m1.is_empty());
        assert!(group.m5.is_empty());
        assert!(group.m10.is_empty());
        assert!(group.m30.is_empty());
        assert!(group.h1.is_empty());
        assert!(group.h12.is_empty());
        assert!(group.d1.is_empty());
        assert!(group.d7.is_empty());
        assert!(group.d30.is_empty());
    }

    #[test]
    fn test_get_mut_s5() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        group.get_mut(&ScrapeInterval::S5).push(42);
        
        assert_eq!(group.s5, vec![42]);
        assert_eq!(group.get(&ScrapeInterval::S5), &vec![42]);
    }

    #[test]
    fn test_get_mut_all_intervals() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        
        group.get_mut(&ScrapeInterval::S5).push(5);
        group.get_mut(&ScrapeInterval::S10).push(10);
        group.get_mut(&ScrapeInterval::S30).push(30);
        group.get_mut(&ScrapeInterval::M1).push(60);
        group.get_mut(&ScrapeInterval::M5).push(300);
        group.get_mut(&ScrapeInterval::M10).push(600);
        group.get_mut(&ScrapeInterval::M30).push(1800);
        group.get_mut(&ScrapeInterval::H1).push(3600);
        group.get_mut(&ScrapeInterval::H12).push(43200);
        group.get_mut(&ScrapeInterval::D1).push(86400);
        group.get_mut(&ScrapeInterval::D7).push(604800);
        group.get_mut(&ScrapeInterval::D30).push(2592000);
        
        assert_eq!(*group.get(&ScrapeInterval::S5), vec![5]);
        assert_eq!(*group.get(&ScrapeInterval::S10), vec![10]);
        assert_eq!(*group.get(&ScrapeInterval::S30), vec![30]);
        assert_eq!(*group.get(&ScrapeInterval::M1), vec![60]);
        assert_eq!(*group.get(&ScrapeInterval::M5), vec![300]);
        assert_eq!(*group.get(&ScrapeInterval::M10), vec![600]);
        assert_eq!(*group.get(&ScrapeInterval::M30), vec![1800]);
        assert_eq!(*group.get(&ScrapeInterval::H1), vec![3600]);
        assert_eq!(*group.get(&ScrapeInterval::H12), vec![43200]);
        assert_eq!(*group.get(&ScrapeInterval::D1), vec![86400]);
        assert_eq!(*group.get(&ScrapeInterval::D7), vec![604800]);
        assert_eq!(*group.get(&ScrapeInterval::D30), vec![2592000]);
    }

    #[test]
    fn test_get_returns_correct_vector() {
        let mut group: GroupByInterval<String> = GroupByInterval::new();
        group.s5.push("test".to_string());
        
        let result = group.get(&ScrapeInterval::S5);
        assert_eq!(result, &vec!["test".to_string()]);
    }

    #[test]
    fn test_iter_returns_all_intervals() {
        let group: GroupByInterval<i32> = GroupByInterval::new();
        let iterations = group.iter();
        
        assert_eq!(iterations.len(), 12);
        assert_eq!(iterations[0].0, ScrapeInterval::S5);
        assert_eq!(iterations[1].0, ScrapeInterval::S10);
        assert_eq!(iterations[2].0, ScrapeInterval::S30);
        assert_eq!(iterations[3].0, ScrapeInterval::M1);
        assert_eq!(iterations[4].0, ScrapeInterval::M5);
        assert_eq!(iterations[5].0, ScrapeInterval::M10);
        assert_eq!(iterations[6].0, ScrapeInterval::M30);
        assert_eq!(iterations[7].0, ScrapeInterval::H1);
        assert_eq!(iterations[8].0, ScrapeInterval::H12);
        assert_eq!(iterations[9].0, ScrapeInterval::D1);
        assert_eq!(iterations[10].0, ScrapeInterval::D7);
        assert_eq!(iterations[11].0, ScrapeInterval::D30);
    }

    #[test]
    fn test_iter_with_data() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        group.s5.push(1);
        group.m1.push(2);
        group.h1.push(3);
        
        let iterations = group.iter();
        
        assert_eq!(*iterations[0].1, vec![1]); // s5
        assert_eq!(*iterations[3].1, vec![2]); // m1
        assert_eq!(*iterations[7].1, vec![3]); // h1
    }

    #[test]
    fn test_merge_empty_groups() {
        let group1: GroupByInterval<i32> = GroupByInterval::new();
        let group2: GroupByInterval<i32> = GroupByInterval::new();
        
        let merged = group1.merge(group2);
        
        assert!(merged.s5.is_empty());
        assert!(merged.s10.is_empty());
        assert!(merged.m1.is_empty());
    }

    #[test]
    fn test_merge_with_data() {
        let mut group1: GroupByInterval<i32> = GroupByInterval::new();
        group1.s5.push(1);
        group1.m1.push(2);
        
        let mut group2: GroupByInterval<i32> = GroupByInterval::new();
        group2.s5.push(3);
        group2.h1.push(4);
        
        let merged = group1.merge(group2);
        
        assert_eq!(merged.s5, vec![1, 3]);
        assert_eq!(merged.m1, vec![2]);
        assert_eq!(merged.h1, vec![4]);
    }

    #[test]
    fn test_merge_all_intervals() {
        let mut group1: GroupByInterval<String> = GroupByInterval::new();
        group1.s5.push("a".to_string());
        group1.s10.push("b".to_string());
        group1.s30.push("c".to_string());
        group1.m1.push("d".to_string());
        group1.m5.push("e".to_string());
        group1.m10.push("f".to_string());
        group1.m30.push("g".to_string());
        group1.h1.push("h".to_string());
        group1.h12.push("i".to_string());
        group1.d1.push("j".to_string());
        group1.d7.push("k".to_string());
        group1.d30.push("l".to_string());
        
        let mut group2: GroupByInterval<String> = GroupByInterval::new();
        group2.s5.push("1".to_string());
        group2.s10.push("2".to_string());
        group2.s30.push("3".to_string());
        group2.m1.push("4".to_string());
        group2.m5.push("5".to_string());
        group2.m10.push("6".to_string());
        group2.m30.push("7".to_string());
        group2.h1.push("8".to_string());
        group2.h12.push("9".to_string());
        group2.d1.push("10".to_string());
        group2.d7.push("11".to_string());
        group2.d30.push("12".to_string());
        
        let merged = group1.merge(group2);
        
        assert_eq!(merged.s5.len(), 2);
        assert_eq!(merged.s10.len(), 2);
        assert_eq!(merged.s30.len(), 2);
        assert_eq!(merged.m1.len(), 2);
        assert_eq!(merged.m5.len(), 2);
        assert_eq!(merged.m10.len(), 2);
        assert_eq!(merged.m30.len(), 2);
        assert_eq!(merged.h1.len(), 2);
        assert_eq!(merged.h12.len(), 2);
        assert_eq!(merged.d1.len(), 2);
        assert_eq!(merged.d7.len(), 2);
        assert_eq!(merged.d30.len(), 2);
    }

    #[test]
    fn test_merge_preserves_order() {
        let mut group1: GroupByInterval<i32> = GroupByInterval::new();
        group1.s5.push(1);
        group1.s5.push(2);
        
        let mut group2: GroupByInterval<i32> = GroupByInterval::new();
        group2.s5.push(3);
        group2.s5.push(4);
        
        let merged = group1.merge(group2);
        
        assert_eq!(merged.s5, vec![1, 2, 3, 4]);
    }

    #[test]
    fn test_merge_does_not_modify_original() {
        let mut group1: GroupByInterval<i32> = GroupByInterval::new();
        group1.s5.push(1);
        
        let mut group2: GroupByInterval<i32> = GroupByInterval::new();
        group2.s5.push(2);
        
        let original_group1_len = group1.s5.len();
        let _ = group1.merge(group2);
        
        assert_eq!(group1.s5.len(), original_group1_len);
    }

    #[test]
    fn test_into_iter() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        group.s5.push(5);
        group.m1.push(60);
        group.h1.push(3600);
        
        let mut iter = group.into_iter();
        
        let first = iter.next().unwrap();
        assert_eq!(first.0, ScrapeInterval::S5);
        assert_eq!(first.1, vec![5]);
        
        let second = iter.next().unwrap();
        assert_eq!(second.0, ScrapeInterval::S10);
        assert!(second.1.is_empty());
    }

    #[test]
    fn test_into_iter_consumes_all() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        group.s5.push(1);
        group.d30.push(2);
        
        let count = group.into_iter().count();
        assert_eq!(count, 12);
    }

    #[test]
    fn test_clone() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        group.s5.push(42);
        group.m1.push(100);
        
        let cloned = group.clone();
        
        assert_eq!(cloned.s5, group.s5);
        assert_eq!(cloned.m1, group.m1);
    }

    #[test]
    fn test_multiple_pushes_same_interval() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        
        for i in 0..10 {
            group.get_mut(&ScrapeInterval::S5).push(i);
        }
        
        assert_eq!(group.s5.len(), 10);
        assert_eq!(group.s5, vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9]);
    }

    #[test]
    fn test_merge_with_multiple_items() {
        let mut group1: GroupByInterval<i32> = GroupByInterval::new();
        for i in 0..100 {
            group1.get_mut(&ScrapeInterval::S5).push(i);
        }
        
        let mut group2: GroupByInterval<i32> = GroupByInterval::new();
        for i in 100..200 {
            group2.get_mut(&ScrapeInterval::S5).push(i);
        }
        
        let merged = group1.merge(group2);
        
        assert_eq!(merged.s5.len(), 200);
        assert_eq!(merged.s5[0], 0);
        assert_eq!(merged.s5[99], 99);
        assert_eq!(merged.s5[100], 100);
        assert_eq!(merged.s5[199], 199);
    }

    #[test]
    fn test_with_complex_types() {
        #[derive(Debug, Clone, PartialEq)]
        struct TestData {
            id: u32,
            name: String,
        }
        
        let mut group: GroupByInterval<TestData> = GroupByInterval::new();
        
        group.get_mut(&ScrapeInterval::S5).push(TestData {
            id: 1,
            name: "test1".to_string(),
        });
        
        group.get_mut(&ScrapeInterval::M1).push(TestData {
            id: 2,
            name: "test2".to_string(),
        });
        
        assert_eq!(group.s5.len(), 1);
        assert_eq!(group.s5[0].id, 1);
        assert_eq!(group.m1[0].name, "test2");
    }

    #[test]
    fn test_iter_references_are_valid() {
        let mut group: GroupByInterval<i32> = GroupByInterval::new();
        group.s5.push(42);
        
        let iterations = group.iter();
        let s5_ref = iterations[0].1;
        
        assert_eq!(s5_ref, &vec![42]);
        assert_eq!(s5_ref[0], 42);
    }

    #[test]
    fn test_merge_chain() {
        let mut group1: GroupByInterval<i32> = GroupByInterval::new();
        group1.s5.push(1);
        
        let mut group2: GroupByInterval<i32> = GroupByInterval::new();
        group2.s5.push(2);
        
        let mut group3: GroupByInterval<i32> = GroupByInterval::new();
        group3.s5.push(3);
        
        let merged = group1.merge(group2).merge(group3);
        
        assert_eq!(merged.s5, vec![1, 2, 3]);
    }
}
