#[cfg(test)]
mod tests {
    use backend::*;
    #[test]
    fn test_bitarray() {
        let mut a = Tools::BitArray::zero();
        a.set(10, true);
        println!("{:?}", &a);
        assert_eq!(a.get(10), true);
        assert_eq!(a.get(9), false);
        assert_eq!(a.get(11), false);
        a.set(10, false);
        a.set(9, true);
        println!("{:?}", &a);
        assert_eq!(a.get(10), false);
        assert_eq!(a.get(9), true);
        assert_eq!(a.get(11), false);
        a.set(101, true);
        a.set(0, true);
        println!("{:?}", &a);
        assert_eq!(a.get(100), false);
        assert_eq!(a.get(101), true);
        assert_eq!(a.get(102), false);
    }
    #[test]
    fn test_load() {
        let a = Subject::Subject::load("../data.json");
        let b = Subject::Subject::zipped_json(&a);
        println!("{:}", &b);
    }

    #[test]
    fn test_combination() {
        let subject_vec = Subject::Subject::load("../data.json");

        let combinator = Tools::SubjectCombinator::new(subject_vec.clone());
        let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
        let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
        let mut sel_subs = vec!["HL320".to_string()];
        let ans = combinator.combinate_subjects(&fix_subs, &mut req_subs, &mut sel_subs);

        assert_eq!(ans.unwrap().unwrap().len(), 12);
    }
}