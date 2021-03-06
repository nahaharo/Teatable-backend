use std::collections::HashMap;

use super::Subject::*;

use std::fmt;
use packed_simd::{u64x4};

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

type SingleCombination = Vec<usize>;
/// Struct for combinate subjects with conditions.
/// 
/// # Algorithm for check conflict: 
/// 1. Assign array index to each class based on time that occupied. \
///     ex: If two class has same time table, it has same array index.
/// 2. For each time(table) that has been counted as array index, check conflict each other.
/// 3. Save these conflict boolean to BitArray.
/// 4. When we need to check conflict, we call BitArray corresponding to index then masking using SIMD(Performance!!)
pub struct SubjectCombinator {
    // conflict_array: pre-calculated conflict boolean.
    conflict_array: [BitArray; 256],
    // code_to_subject: mapping code and class number to index in "conflict_array".
    // Shape of {class code: [conflict_array idx of each class]}.
    code_to_subject: HashMap<String, Vec<u8>>,
    // code_to_num: mapping code and class number to index in "Subject vector".
    code_to_num: HashMap<String, Vec<usize>>,
    // subjects: Storage for Subjects.
    subjects: Vec<Subject>,

    // object pool for SingleCombination.
    obj_pool: Pool<Vec<usize>>,
    comb_pool: Pool<Vec<RcRecycled<SingleCombination>>>,
    bit_pool: Pool<Vec<BitArray>>,
}

impl Clone for SubjectCombinator {
    fn clone(&self) -> Self {
        SubjectCombinator {
            conflict_array: self.conflict_array.clone(),
            code_to_subject: self.code_to_subject.clone(),
            code_to_num: self.code_to_num.clone(),
            subjects: self.subjects.clone(),
            obj_pool: pool().with(StartingSize(256)).with(Supplier(|| Vec::with_capacity(30))).build(),
            comb_pool: pool().with(StartingSize(2)).with(Supplier(|| Vec::with_capacity(1000))).build(),
            bit_pool: pool().with(StartingSize(2)).with(Supplier(|| Vec::with_capacity(1000))).build()
        }
    }
}

impl SubjectCombinator {
    pub fn new(subs: Vec<Subject>) -> Self {
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
            obj_pool: pool().with(StartingSize(256)).with(Supplier(|| Vec::with_capacity(30))).build(),
            comb_pool: pool().with(StartingSize(2)).with(Supplier(|| Vec::with_capacity(1000))).build(),
            bit_pool: pool().with(StartingSize(2)).with(Supplier(|| Vec::with_capacity(1000))).build()
        }
    }

    pub fn combinate_subjects(&self, fixsubs: &Vec<(String, /*Index, not class number*/usize)>, reqsubs: &mut Vec<String>, selsubs: &mut Vec<String>)
     -> Result<Option<Recycled<Vec<RcRecycled<SingleCombination>>>>, &str> {
        // first, sort reqired subjects(reqsubs) and selected subjects(selsubs) by use key as number of classes (which sharing subject code)
        // this reduce execution time
        let mut req_no_sub_flag = false;
        reqsubs.sort_unstable_by_key(|x| match self.code_to_subject.get(x) {
            Some(t) => t.len(),
            None => {
                req_no_sub_flag = true;
                0
            }
        } );
        if req_no_sub_flag {
            return Err("Invalid required subject");
        }

        let mut sel_no_sub_flag = false;
        selsubs.sort_unstable_by_key(|x| match self.code_to_subject.get(x) {
            Some(t) => t.len(),
            None => {
                sel_no_sub_flag = true;
                0
            }
        } );
        if sel_no_sub_flag {
            return Err("Invalid selected subject");
        }

        let mut fix_subs = self.obj_pool.new_rc();
        let mut fix_mask = BitArray::zero();
        // in this loop, we findout fixed subjects from code_to_subject, and check are these in conflict.
        for (sub_code, class_idx) in fixsubs.iter() {
            let idx: u8 = match self.code_to_subject.get(sub_code) {
                Some(t) => t[*class_idx as usize],
                None => return Err("Invalid fix subject")
            } as u8;
            fix_subs.push(self.code_to_num.get(sub_code).unwrap()[*class_idx]);
            if fix_mask.get(idx) {return Ok(None)}
            fix_mask.set(idx, true);
        }

        let mut sub_comb_list = self.comb_pool.new();
        sub_comb_list.push(fix_subs);
        let mut sub_mask_list = self.bit_pool.new();
        sub_mask_list.push(fix_mask.clone());

        // loop for reqired subjects
        for req_code in reqsubs.iter() {  // for each subject( = subject code)  
            let mut is_added = false;

            let mut tmp_req_comb_list = self.comb_pool.new();
            let mut tmp_req_mask_list = self.bit_pool.new();

            let req_subs = self.code_to_subject.get(req_code).unwrap();
            let req_subs_idxs = self.code_to_num.get(req_code).unwrap();
            for (combined_subs, bit) in sub_comb_list.iter().zip(sub_mask_list.iter()) { // for each class with same code
                for (class_idx, bit_idx) in req_subs.iter().enumerate() { // for each time block in class. subs: Vec<(String, usize)>, bit: BitArray
                    let sub_conflict_bit: u64x4 =  self.conflict_array[*bit_idx as usize].clone().into();
                    let combined_bit: u64x4 = bit.clone().into(); // current mask
                    let m = (sub_conflict_bit | combined_bit).eq(sub_conflict_bit ^ combined_bit).all(); // Check if is it conflict
                    if m {
                        is_added = true;
                        let mut new_sub = combined_subs.clone();
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

            if !is_added {
                return Ok(None)
            }
        }
        
        for sel_code in selsubs.iter() {
            let sel_subs = self.code_to_subject.get(sel_code).unwrap();
            let sel_subs_idxs = self.code_to_num.get(sel_code).unwrap();
            for idx in 0..sub_comb_list.len() { 
                for (class_idx, bit_idx) in sel_subs.iter().enumerate() { // subs: Vec<(String, usize)>, bit: BitArray
                    let combined_subs = &sub_comb_list[idx];
                    let bit = &sub_mask_list[idx];
                    let sub_conflict_bit: u64x4 =  self.conflict_array[*bit_idx as usize].clone().into();
                    let combined_bit: u64x4 = bit.clone().into(); // current mask
                    let m = (sub_conflict_bit | combined_bit).eq(sub_conflict_bit ^ combined_bit).all();
                    if m {
                        let mut new_sub = combined_subs.clone();
                        let mut new_bit = bit.clone();
                        new_sub.push(sel_subs_idxs[class_idx]);
                        new_bit.set(*bit_idx, true);
                        sub_comb_list.push(new_sub);
                        sub_mask_list.push(new_bit);
                    }
                }
            }
        }

        if sub_comb_list.len() == 0 {
            Ok(None)
        }
        else {
            sub_comb_list.reverse();
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