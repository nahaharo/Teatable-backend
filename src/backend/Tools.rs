use std::collections::HashMap;

use crate::backend::DataIO::Subject;

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

pub fn comb_sub(subdata: &HashMap<String, Vec<Subject>>,fixsubs: &Vec<(String, /*Index, not class number*/usize)>, reqsubs: &Vec<String>, selsubs: &Vec<String>) -> Result<Option<Vec<Vec<u32>>>, String>
{
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
                return Err(String::from("Invalid fix class number!"));
            }
        }
        else
        {
            return Err(String::from("Invalid fixed class code!"));
        }
    }

    let mut sub_comb_list = vec![fix_sub_vec];
    let mut sub_comb_bits = vec![fix_bit];

    for req_code in reqsubs.iter()
    {
        if let Some(req_subs) = subdata.get(req_code)
        {   
            let mut flag: bool = false;//check if require subject are included
            let mut req_comb_dump: Vec<Vec<u32>> = Vec::new();
            let mut req_bits_dump: Vec<[u64; 5]> = Vec::new();
            for req_sub in req_subs.iter()
            { 
                for (comb, bit) in sub_comb_list.iter().zip(sub_comb_bits.iter())
                {
                    match merge_time_bit(&req_sub.time_bit, &bit)
                    {
                        Some(t) => {
                            flag = true;
                            let mut c = comb.clone();
                            c.push(req_sub.number);
                            req_comb_dump.push(c);
                            req_bits_dump.push(t);
                        }
                        None => {}
                    }
                }
            }
            sub_comb_list = req_comb_dump;
            sub_comb_bits = req_bits_dump;
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
    
    for sel_code in selsubs.iter()
    {
        if let Some(sel_subs) = subdata.get(sel_code)
        {
            let mut sel_comb_dump: Vec<Vec<u32>> = Vec::new();
            let mut sel_bits_dump: Vec<[u64; 5]> = Vec::new();
            for sel_sub in sel_subs.iter()
            { 
                for (comb, bit) in sub_comb_list.clone().iter().zip(sub_comb_bits.clone().iter())
                {
                    match merge_time_bit(&sel_sub.time_bit, &bit)
                    {
                        Some(t) => {
                            let mut c = comb.clone();
                            c.push(sel_sub.number);
                            sel_comb_dump.push(c);
                            sel_bits_dump.push(t);
                        }
                        None => {}
                    }
                }
            }
            sub_comb_list.append(&mut sel_comb_dump);
            sub_comb_bits.append(&mut sel_bits_dump);
        }
        else
        {
            return Err(String::from("Invalid selected class code!"));
        }
    }

    let idxs : Vec<usize>  = (0..sub_comb_list.len()).collect();
    let mut ans: Vec<(Vec<u32>, usize)> = sub_comb_list.into_iter().zip(idxs.into_iter()).collect();

    ans.sort_unstable_by_key(|(_, b)| hamming_weight(&sub_comb_bits[b.clone()]));

    let (mut ans, _): (Vec<Vec<u32>>, Vec<usize>)= ans.into_iter().unzip();
    ans.reverse();
    Ok(Some(ans))
}


