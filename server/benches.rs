use criterion::{black_box, criterion_group, criterion_main, Criterion};
use backend::*;
use std::collections::HashMap;

mod crawler;

pub fn bench_combination(c: &mut Criterion) {
    let a = crawler::SubjectResponse::from_file("foo.txt".to_string());

    let subject_vec = a.to_subject_vector();
    let combinator = Tools::SubjectCombinator::new(&subject_vec);
    let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
    let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
    let mut sel_subs = vec!["HL320".to_string()];
    c.bench_function("New", |b| b.iter(|| 
        combinator.comb_sub(black_box(&fix_subs), black_box(&mut req_subs), black_box(&mut sel_subs))
    ));

    let mut subjects = HashMap::new();
    for sub in subject_vec.into_iter() {
        subjects.entry(sub.code.clone()).or_insert_with(Vec::new).push(sub);
    }

    c.bench_function("Old", |b| b.iter(|| 
        Tools::comb_sub(&subjects, black_box(&fix_subs), black_box(&mut req_subs), black_box(&mut sel_subs))
    ));
}

criterion_group!(benches, bench_combination);
criterion_main!(benches);