use std::collections::HashMap;

use super::Subject::*;

use std::fmt;
use packed_simd::{u64x4};
use std::rc::Rc;

use lifeguard::*;

#[derive(Copy, Clone)]
pub struct BitArray {
    elem: [u64; 4]
}

impl BitArray {
    #[inline(always)]
    pub fn get(&self, i: u8) -> bool {
        (self.elem[(i/64) as usize]>>(i%64))%2 != 0
    }
    #[inline(always)]
    pub fn set(&mut self, i: u8, flag: bool) {
        if flag {
            self.elem[(i/64) as usize] |= (1 as u64) << (i%64);
        }
        else {
            self.elem[(i/64) as usize] &= !((1 as u64) << (i%64));
        }
        
    }
    #[inline(always)]
    pub fn zero() -> Self {
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

#[derive(Clone)]
pub struct SubjectCombinator {
    conflict_array: [BitArray; 256],
    code_to_subject: HashMap<String, Vec<u8>>,// {class code: [conflict_array idx of each class]}
    code_to_num: HashMap<String, Vec<usize>>,
    subjects: Vec<Subject>,

    pool: Rc<Pool<Vec<usize>>>,
}

impl SubjectCombinator {
    pub fn new(subs: Vec<Subject>, obj_pool: Rc<Pool<Vec<usize>>>) -> Self {
        let mut subject_map: HashMap<String, Vec<&Subject>> = HashMap::new();
        let mut code_to_num: HashMap<String, Vec<usize>> = HashMap::new();
        let mut time_map: HashMap<[u64;5], u8> = HashMap::new();
        let mut idx_maps: HashMap<String, Vec<u8>> = HashMap::new();
        let mut idx: u8 = 0;
        for (subs_idx, e) in subs.iter().enumerate() {
            match subject_map.get_mut(&e.code) {
                Some(v) => v.push(e),
                None => { subject_map.insert(e.code.clone(), vec![e]); }
            }
            if !time_map.contains_key(&e.time_bit) {
                time_map.insert(e.time_bit.clone(), idx);
                idx += 1;
            }
            match code_to_num.get_mut(&e.code) {
                Some(v) => v.push(subs_idx),
                None => { code_to_num.insert(e.code.clone(), vec![subs_idx]); }
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
            code_to_num: code_to_num,
            subjects: subs,
            pool: obj_pool
        }
    }

    pub fn combinate_subjects(&self, fixsubs: &Vec<(String, /*Index, not class number*/usize)>, reqsubs: &mut Vec<String>, selsubs: &mut Vec<String>)
     -> Result<Option<Vec<Recycled<Vec<usize>>>>, &str> {
        reqsubs.sort_unstable_by_key(|x| self.code_to_subject.get(x).unwrap_or(&Vec::new()).len());
        selsubs.sort_unstable_by_key(|x| self.code_to_subject.get(x).unwrap_or(&Vec::new()).len());

        let mut fix_subs = self.pool.new();
        let mut fix_mask = BitArray::zero();
        for (sub_code, class_idx) in fixsubs.iter() {
            let idx: u8 = match self.code_to_subject.get(sub_code) {
                Some(t) => t[*class_idx as usize],
                None => return Err("Invalid fix subject")
            } as u8;
            fix_subs.push(self.code_to_num.get(sub_code).unwrap()[*class_idx]);
            if fix_mask.get(idx) {return Ok(None)}
            fix_mask.set(idx, true);
        }

        let mut sub_comb_list = Vec::with_capacity(20);
        sub_comb_list.push(fix_subs);
        let mut sub_mask_list = Vec::with_capacity(20);
        sub_mask_list.push(fix_mask.clone());

        for req_code in reqsubs.iter() {   
            let mut is_added = false;

            let mut tmp_req_comb_list = Vec::with_capacity(sub_comb_list.len()+10);
            let mut tmp_req_mask_list = Vec::with_capacity(sub_comb_list.len()+10);

            if let Some(req_subs) = self.code_to_subject.get(req_code) { // req_subs: Vec<usize>
                let req_subs_idxs = self.code_to_num.get(req_code).unwrap();
                for (class_idx, bit_idx) in req_subs.iter().enumerate() { 
                    for (combined_subs, bit) in sub_comb_list.iter().zip(sub_mask_list.iter()) { // subs: Vec<(String, usize)>, bit: BitArray
                        let sub_conflict_bit: u64x4 =  self.conflict_array[*bit_idx as usize].clone().into();
                        let combined_bit: u64x4 = bit.clone().into(); // current mask
                        let m = (sub_conflict_bit | combined_bit).eq(sub_conflict_bit ^ combined_bit).all();
                        if m {
                            is_added = true;
                            let mut new_sub = self.pool.new_from(combined_subs.iter().cloned());
                            let mut new_bit = bit.clone();
                            new_sub.push(req_subs_idxs[class_idx]);
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
                let sel_subs_idxs = self.code_to_num.get(sel_code).unwrap();
                for (class_idx, bit_idx) in sel_subs.iter().enumerate() { 
                    for idx in 0..sub_comb_list.len() { // subs: Vec<(String, usize)>, bit: BitArray
                        let combined_subs = &sub_comb_list[idx];
                        let bit = &sub_mask_list[idx];
                        let sub_conflict_bit: u64x4 =  self.conflict_array[*bit_idx as usize].clone().into();
                        let combined_bit: u64x4 = bit.clone().into(); // current mask
                        let m = (sub_conflict_bit | combined_bit).eq(sub_conflict_bit ^ combined_bit).all();
                        if m {
                            let mut new_sub = self.pool.new_from(combined_subs.iter().cloned());
                            let mut new_bit = bit.clone();
                            new_sub.push(sel_subs_idxs[class_idx]);
                            new_bit.set(*bit_idx, true);
                            sub_comb_list.push(new_sub);
                            sub_mask_list.push(new_bit);
                        }
                    }
                }
            }
            else { return Err("Invalid selected subject") }
        }

        if sub_comb_list.len() == 0 {
            Ok(None)
        }
        else {
            Ok(Some(
                sub_comb_list
                // sub_comb_list.into_iter().map(
                //     |x| x.detach()
                // ).collect()
            ))
        }
    }
}

unsafe impl Send for SubjectCombinator {}
unsafe impl Sync for SubjectCombinator {}

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

fn hamming_weight(x: &[u64; 5]) -> u32
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