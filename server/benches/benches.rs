#![feature(test)]
extern crate test;

use test::{Bencher, black_box};
use backend::*;
use std::collections::HashMap;

#[bench]
fn old_combination(b: &mut Bencher) {
    let a = crawler::SubjectResponse::from_file("foo.txt".to_string());

    let subject_vec = a.to_subject_vector();
    // let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
    // let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
    // let mut sel_subs = vec!["HL320".to_string()];

    let fix_subs = black_box(vec![]);
    let mut req_subs = black_box(vec!["SE102a".to_string(), "SE105a".to_string(), "SE106".to_string(), "SE108a".to_string(), "SE109".to_string(), "SE111".to_string(), "SE112".to_string(), "SE118".to_string()]);
    let mut sel_subs = black_box(vec![]);

    let mut subjects = HashMap::new();
    for sub in subject_vec.clone().into_iter() {
        subjects.entry(sub.code.clone()).or_insert_with(Vec::new).push(sub);
    }

    b.iter(|| {
        Tools::comb_sub(&subjects, &fix_subs, &mut req_subs, &mut sel_subs);
    });
}

#[bench]
fn new_combination(b: &mut Bencher) {
    let a =  crawler::SubjectResponse::from_file("foo.txt".to_string());

    let subject_vec = a.to_subject_vector();
    let combinator = Tools::SubjectCombinator::new(&subject_vec);
    // let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
    // let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
    // let mut sel_subs = vec!["HL320".to_string()];

    let fix_subs = black_box(vec![]);
    let mut req_subs = black_box(vec!["SE102a".to_string(), "SE105a".to_string(), "SE106".to_string(), "SE108a".to_string(), "SE109".to_string(), "SE111".to_string(), "SE112".to_string(), "SE118".to_string()]);
    let mut sel_subs = black_box(vec![]);

    b.iter(|| {
        combinator.comb_sub(&fix_subs, &mut req_subs, &mut sel_subs);
    });
}