use std::collections::HashMap;

use super::Subject::*;


use min_max_heap::MinMaxHeap;
use std::cmp::Ordering;
use std::time::{SystemTime};
use std::fmt;
use packed_simd::{u64x4};

#[derive(Copy, Clone)]
pub struct BitArray {
    elem: [u64; 4]
}

impl BitArray {
    #[inline(always)]
    fn get(&self, i: u8) -> bool {
        (self.elem[(i>>6) as usize]>>(i&63))&1 != 0
    }
    #[inline(always)]
    fn set(&mut self, i: u8, flag: bool) {
        if flag {
            self.elem[(i>>6) as usize] |= (1 as u64) << (i&63);
        }
        else {
            self.elem[(i>>6) as usize] &= !((1 as u64) << (i&63));
        }
        
    }
    #[inline(always)]
    fn zero() -> Self {
        BitArray::from([
            0 as u64,0 as u64,0 as u64,0 as u64
        ])
    }
}

impl From<[u64; 4]> for BitArray {
    #[inline(always)]
    fn from(a: [u64; 4]) -> Self{
        BitArray {
            elem: a
        }
    }
}

impl Into<[u64; 4]> for BitArray {
    #[inline(always)]
    fn into(self) -> [u64; 4] {
        self.elem
    }
}

impl Into<u64x4> for BitArray {
    #[inline(always)]
    fn into(self) -> u64x4 {
        u64x4::from_slice_unaligned(&self.elem)
    }
} 

impl fmt::Debug for BitArray {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let elem = self.elem;
        write!(f, "{:64b}|{:64b}\n{:64b}|{:64b}", 
        elem[0], elem[1], elem[2], elem[3])
    }
}

pub struct SubjectCombinator {
    conflict_array: [BitArray; 256],
    code_to_subject: HashMap<String, Vec<u8>>,// {class code: [conflict_array idx of each class]}
}

impl SubjectCombinator {
    pub fn new(subs: &Vec<Subject>) -> Self {
        let mut subject_map: HashMap<String, Vec<&Subject>> = HashMap::new();
        let mut time_map: HashMap<[u64;5], u8> = HashMap::new();
        let mut idx_maps: HashMap<String, Vec<u8>> = HashMap::new();
        let mut idx: u8 = 0;
        for e in subs.iter() {
            match subject_map.get_mut(&e.code) {
                Some(v) => v.push(e),
                None => { subject_map.insert(e.code.clone(), vec![e]); }
            }
            if !time_map.contains_key(&e.time_bit) {
                time_map.insert(e.time_bit.clone(), idx);
                idx += 1;
            }
        }

        for (k, e) in subject_map.iter_mut() {
            e.sort_unstable_by_key(|x| x.class_num);
            idx_maps.insert(k.clone(), e.iter().map(|x| *time_map.get(&x.time_bit).unwrap() ).collect());
        }

        let mut conflict_bit = [BitArray::zero(); 256];
        for (k1, e1) in time_map.iter() {
            for (k2, e2) in time_map.iter() {
                if is_conflict(k1, k2) {
                    conflict_bit[*e1 as usize].set(*e2, true);
                    conflict_bit[*e2 as usize].set(*e1, true);
                }
            }
        }
        SubjectCombinator {
            conflict_array: conflict_bit,
            code_to_subject: idx_maps,
        }
    }

    pub fn comb_sub(&self, fixsubs: &Vec<(String, /*Index, not class number*/usize)>, reqsubs: &mut Vec<String>, selsubs: &mut Vec<String>)
     -> Result<Option<Vec<Vec<(String, usize)>>>, &str> {
        reqsubs.sort_unstable_by_key(|x| self.code_to_subject.get(x).unwrap_or(&Vec::new()).len());
        selsubs.sort_unstable_by_key(|x| self.code_to_subject.get(x).unwrap_or(&Vec::new()).len());
        let mut fix_mask = BitArray::zero();
        for (sub_code, class_idx) in fixsubs.iter() {
            let idx: u8 = match self.code_to_subject.get(sub_code) {
                Some(t) => t[*class_idx as usize],
                None => return Err("Invalid fix subject")
            } as u8;
            if fix_mask.get(idx) {return Ok(None)}
            fix_mask.set(idx, true);
        }

        let mut sub_comb_list = vec![fixsubs.clone()];
        let mut sub_mask_list = vec![fix_mask.clone()];
        for req_code in reqsubs.iter() {   
            let mut is_added = false;
            let mut tmp_req_comb_list = Vec::new();
            let mut tmp_req_mask_list = Vec::new();
            if let Some(req_subs) = self.code_to_subject.get(req_code) { // req_subs: Vec<usize>
                for (class_idx, bit_idx) in req_subs.iter().enumerate() { 
                    for idx in 0..sub_comb_list.len() { // subs: Vec<(String, usize)>, bit: BitArray
                        let combined_subs = &sub_comb_list[idx];
                        let bit = &sub_mask_list[idx];
                        let sub_conflict_bit: u64x4 =  self.conflict_array[*bit_idx as usize].clone().into();
                        let combined_bit: u64x4 = bit.clone().into(); // current mask
                        let m = (sub_conflict_bit | combined_bit).eq(sub_conflict_bit ^ combined_bit).all();
                        if m {
                            is_added = true;
                            let mut new_sub = combined_subs.clone();
                            let mut new_bit = bit.clone();
                            new_sub.push((req_code.clone(), class_idx));
                            new_bit.set(*bit_idx, true);
                            tmp_req_comb_list.push(new_sub);
                            tmp_req_mask_list.push(new_bit);
                        }
                    }
                }
                sub_comb_list = tmp_req_comb_list;
                sub_mask_list = tmp_req_mask_list;
            }
            else { return Err("Invalid require subject") }
            if !is_added {
                return Ok(None)
            }
        }
        
        for sel_code in selsubs.iter() {
            if let Some(sel_subs) = self.code_to_subject.get(sel_code) {
                for (class_idx, bit_idx) in sel_subs.iter().enumerate() { 
                    for idx in 0..sub_comb_list.len() { // subs: Vec<(String, usize)>, bit: BitArray
                        let combined_subs = &sub_comb_list[idx];
                        let bit = &sub_mask_list[idx];
                        let sub_conflict_bit: u64x4 =  self.conflict_array[*bit_idx as usize].clone().into();
                        let combined_bit: u64x4 = bit.clone().into(); // current mask
                        let m = (sub_conflict_bit | combined_bit).eq(sub_conflict_bit ^ combined_bit).all();
                        if m {
                            let mut new_sub = combined_subs.clone();
                            let mut new_bit = bit.clone();
                            new_sub.push((sel_code.clone(), class_idx));
                            new_bit.set(*bit_idx, true);
                            sub_comb_list.push(new_sub);
                            sub_mask_list.push(new_bit);
                        }
                    }
                }
            }
            else { return Err("Invalid selected subject") }
        }

        Ok(Some( 
            sub_comb_list
        ))
    }
}

fn is_conflict(a: &[u64; 5], b: &[u64; 5]) -> bool
{
    let mut tmp: [u64; 5] = [0,0,0,0,0];
    let mut k = false;
    for i in 0..5 as usize
    {
        if a[i] | b[i] != a[i] ^ b[i]//conflict occur
        {
            k = true;
        }
        tmp[i] = a[i] + b[i];
    }
    k
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
    let mut k = false;
    for i in 0..5 as usize
    {
        if a[i] | b[i] != a[i] + b[i]//conflict occur
        {
            k = true;
        }
        tmp[i] = a[i] + b[i];
    }
    if k {None}
    else {Some(tmp)}
}

#[deprecated(
    since = "2020",
    note = "Do not use this to generate json."
)]
pub fn comb_sub(subdata: &HashMap<String, Vec<Subject>>,fixsubs: &Vec<(String, /*Index, not class number*/usize)>, reqsubs: &mut Vec<String>, selsubs: &mut Vec<String>)
     -> Result<Option<Vec<Vec<u32>>>, String>
{
    let now = SystemTime::now();

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

    let mut i = 0;

    let mut sub_comb_list = vec![Subs{sub_nums: fix_sub_vec, time_bit: fix_bit, factor: 0}];

    for req_code in reqsubs.iter()
    {
        if let Some(req_subs) = subdata.get(req_code)
        {   
            let mut flag: bool = false;//check if require subject are included
            let mut req_comb_dump: Vec<Subs> = Vec::new();
            for req_sub in req_subs.iter()
            { 
                if i%20 == 0
                {
                    match now.elapsed() {
                        Ok(elapsed) => {
                            if elapsed.as_secs()> 10
                            {
                                return Err(String::from("Function Timeout!"));
                            }
                        }
                        Err(_) => {
                            // an error occurred!
                            return Err(String::from("Function Timeout!"));
                        }
                    }
                }
                i+=1;

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

                if i%20 == 0
                {
                    match now.elapsed() {
                        Ok(elapsed) => {
                            if elapsed.as_secs()> 10
                            {
                                return Err(String::from("Function Timeout!"));
                            }
                        }
                        Err(_) => {
                            // an error occurred!
                            return Err(String::from("Function Timeout!"));
                        }
                    }
                }
                i+=1;

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


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_bitarray() {
        let mut a = BitArray::zero();
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
}