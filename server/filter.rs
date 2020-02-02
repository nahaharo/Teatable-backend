pub fn filter_query(req: &Vec<String>, sel: &Vec<String>) -> bool{
    if req.len() > 10 || sel.len() > 10 {
        return false
    }
    let a: bool = req.iter().fold(false, |x, y| {
        if y.get(2..5)==Some("900") || y == "HL471" || y == "HL302" {
            true || x
        }
        else {
            false || x
        }
    });
    if a {
        return false
    }
    let a: bool = sel.iter().fold(false, |x, y| {
        if y.get(2..5)==Some("900") || y == "HL471" || y == "HL302" {
            true || x
        }
        else {
            false || x
        }
    });
    if a {
        return false
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_filter() {
        let mut req = vec!["SE102".to_string()];
        let mut sel = vec!["SE102".to_string()];
        assert_eq!(filter_query(&req, &sel), true);
        req.push("BS900".to_string());
        assert_eq!(filter_query(&req, &sel), false);
        req.pop();
        for _ in 0..15 {
            req.push("SE102".to_string());
        }
        assert_eq!(filter_query(&req, &sel), false);
        for _ in 0..15 {
            req.pop();
        }
        assert_eq!(filter_query(&req, &sel), true);
        for _ in 0..15 {
            sel.push("SE102".to_string());
        }
        assert_eq!(filter_query(&req, &sel), false);
        for _ in 0..15 {
            sel.pop();
        }
        assert_eq!(filter_query(&req, &sel), true);
    }
}