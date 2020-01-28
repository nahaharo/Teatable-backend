#![feature(test)]
extern crate test;

use test::{Bencher, black_box};
use backend::*;
use std::collections::HashMap;

#[bench]
fn old_combination(b: &mut Bencher) {
    let subject_vec = Subject::Subject::load("data.json");

    // let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
    // let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
    // let mut sel_subs = vec!["HL320".to_string()];

    let fix_subs = black_box(vec![]);
    let mut req_subs = black_box(vec!["SE102a", "SE105a", "SE106", "SE108a", "SE109", "SE111", "SE112", "SE118"]);
    let mut sel_subs = black_box(vec![]);

    let mut subjects = HashMap::new();
    for sub in subject_vec.clone().into_iter() {
        subjects.entry(sub.code.to_string()).or_insert_with(Vec::new).push(sub);
    }

    b.iter(|| {
        Tools::comb_sub(&subjects, &fix_subs, &mut req_subs, &mut sel_subs);
    });
}

#[bench]
fn new_combination(b: &mut Bencher) {
    let subject_vec = Subject::Subject::load("data.json");
    
    let combinator = Tools::SubjectCombinator::new(subject_vec.clone());
    // let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
    // let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
    // let mut sel_subs = vec!["HL320".to_string()];

    let fix_subs = black_box(vec![]);
    let mut req_subs = black_box(vec!["SE102a", "SE105a", "SE106", "SE108a", "SE109", "SE111", "SE112", "SE118"]);
    let mut sel_subs = black_box(vec![]);

    b.iter(|| {
        combinator.comb_sub(&fix_subs, &mut req_subs, &mut sel_subs);
    });
}