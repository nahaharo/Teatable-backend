#[cfg(test)]
mod tests {
    use backend::*;
    use std::fs::File;
    use std::io::prelude::*;
    #[test]
    fn test_redis() {
        let q = DB::add_share(&vec![1,2,3]).unwrap();
        let ans = DB::get_share(&q);
        print!("{:?}\n", ans);
        let ans = backend::DB::get_share(&"asdf".to_string());
        print!("{:?}\n", ans);
    }
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
        let a = Subject::Subject::load("data.json");
    }
}