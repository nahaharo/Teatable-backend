use std::collections::HashMap;

use crate::backend::DataIO::Subject;


use min_max_heap::MinMaxHeap;
use std::cmp::Ordering;

#[derive(Clone, Debug)]
struct Subs {
    sub_nums: Vec<u32>,
    time_bit: [u64; 5],
    factor: i32,
}

impl Ord for Subs {
    fn cmp(&self, other: &Subs) -> Ordering {
        other.factor.cmp(&self.factor)
    }
}

impl PartialOrd for Subs {
    fn partial_cmp(&self, other: &Subs)-> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Subs {
    fn eq(&self, other: &Subs) -> bool {
        self.factor == other.factor
    }
}

impl Eq for Subs {}

fn merge_time_bit(a: &[u64; 5], b: &[u64; 5]) -> Option<[u64; 5]>
{
    let mut tmp: [u64; 5] = [0,0,0,0,0];
    for i in 0..5 as usize
    {
        if a[i] | b[i] != a[i] + b[i]//conflict occur
        {
            return None;
        }
        tmp[i] = a[i] + b[i];
    }
    Some(tmp)
}


fn hamming_weight(x: &[u64]) -> u32
{
    const M1:  u64  = 0x5555555555555555;
    const M2:  u64  = 0x3333333333333333;
    const M4:  u64  = 0x0f0f0f0f0f0f0f0f;

    let mut sum:u64 = 0;
    for e in x.iter()
    {
        let mut h = e.clone();
        h -= (h >> 1) & M1;
        h = (h & M2) + ((h >> 2) & M2);
        h = (h + (h >> 4)) & M4;
        h += h >> 8;
        h += h >> 16;
        h += h >> 32;
        h = h & 0x7f;
        sum += h;
    }
    sum as u32
}

pub fn comb_sub(subdata: &HashMap<String, Vec<Subject>>,fixsubs: &Vec<(String, /*Index, not class number*/usize)>, reqsubs: &mut Vec<String>, selsubs: &mut Vec<String>) -> Result<Option<Vec<Vec<u32>>>, String>
{
    reqsubs.sort_unstable_by_key(|x| subdata.get(x).unwrap_or(&Vec::new()).len());
    selsubs.sort_unstable_by_key(|x| subdata.get(x).unwrap_or(&Vec::new()).len());

    let mut fix_bit: [u64; 5] = [0,0,0,0,0];
    let mut fix_sub_vec = Vec::new();
    for (code, num) in fixsubs.iter()
    {
        if let Some(subs) = subdata.get(code)//If subdata.get(code) is not none => code is not valid
        {
            if let Some(sub) = subs.get(num.clone())//If get(code) is not none => num is not valid
            {
                let sub_bit = &sub.time_bit;
                match merge_time_bit(&fix_bit, &sub_bit)
                {
                    Some(t) => {
                        fix_bit = t;
                        fix_sub_vec.push(sub.number);
                    },
                    None => return Ok(None)
                }
            }
            else
            {
                return Err(String::from("Invalid fixed class number!"));
            }
        }
        else
        {
            return Err(String::from("Invalid fixed class code!"));
        }
    }

    let mut sub_comb_list = vec![Subs{sub_nums: fix_sub_vec, time_bit: fix_bit, factor: 0}];
    //let mut sub_comb_bits = vec![fix_bit];

    for req_code in reqsubs.iter()
    {
        if let Some(req_subs) = subdata.get(req_code)
        {   
            let mut flag: bool = false;//check if require subject are included
            let mut req_comb_dump: Vec<Subs> = Vec::new();
            for req_sub in req_subs.iter()
            { 
                for subs in sub_comb_list.iter()
                {
                    match merge_time_bit(&req_sub.time_bit, &subs.time_bit)
                    {
                        Some(t) => {
                            flag = true;
                            let mut c = subs.sub_nums.clone();
                            c.push(req_sub.number);
                            req_comb_dump.push(Subs{sub_nums: c, time_bit: t, factor: 0});
                        }
                        None => {}
                    }
                }
            }
            sub_comb_list = req_comb_dump;
            if !flag
            {
                return Ok(None);
            }
        }
        else
        {
            return Err(String::from("Invalid required class code!"));
        }
    }
    
    let mut sub_comb_list = MinMaxHeap::from(sub_comb_list);
    for sel_code in selsubs.iter()
    {
        if let Some(sel_subs) = subdata.get(sel_code)
        {
            let mut sel_comb_dump: Vec<Subs> = Vec::new();
            for sel_sub in sel_subs.iter()
            { 
                for subs in sub_comb_list.clone().iter()
                {
                    match merge_time_bit(&sel_sub.time_bit, &subs.time_bit)
                    {
                        Some(t) => {
                            let mut c = subs.sub_nums.clone();
                            c.push(sel_sub.number);
                            sel_comb_dump.push(Subs{sub_nums: c, time_bit: t, factor: hamming_weight(&t) as i32});
                        }
                        None => {}
                    }
                }
            }
            sub_comb_list.extend(sel_comb_dump.into_iter());
        }
        else
        {
            return Err(String::from("Invalid selected class code!"));
        }
    }

    let ans = sub_comb_list.into_vec_asc().into_iter().map(|x| x.sub_nums).collect();
    Ok(Some(ans))
}


