#[cfg(test)]
mod tests {
    use crate::*;
    use lazy_static::lazy_static;
    use std::collections::HashMap;
    lazy_static! {
        static ref SUB_MAP: HashMap<String, Vec<backend::DataIO::Subject>> = backend::DataIO::read_csv("./data/data.csv").unwrap();
    }
    #[test]
    fn test_comb_sub() {
        let ans = backend::Tools::comb_sub(&SUB_MAP, &vec![("HL303".to_string(), 31)], &mut vec!["HL203".to_string()], &mut vec!["SE106".to_string()]);
        assert_eq!(ans.unwrap().unwrap().len(), 64);
    }

    #[test]
    fn test_redis() {
        let q = backend::DB::add_share(&vec![1,2,3]).unwrap();
        let ans = backend::DB::get_share(&q);
        print!("{:?}\n", ans);
        let ans = backend::DB::get_share(&"asdf".to_string());
        print!("{:?}\n", ans);
    }
}