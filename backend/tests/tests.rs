#[cfg(test)]
mod tests {
    #[test]
    fn test_redis() {
        let q = backend::DB::add_share(&vec![1,2,3]).unwrap();
        let ans = backend::DB::get_share(&q);
        print!("{:?}\n", ans);
        let ans = backend::DB::get_share(&"asdf".to_string());
        print!("{:?}\n", ans);
    }
}