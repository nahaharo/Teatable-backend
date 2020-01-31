use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::io::BufReader;
use regex;

use lazy_static::lazy_static;
use serde::{Serialize, Deserialize};


lazy_static! {
static ref RE_DATE : regex::Regex = regex::Regex::new(r"([월화수목금토일])+(\w{2}:\w{2})-(\w{2}:\w{2})\((\w{2}\s?-\s?\w*)\)").unwrap();
static ref RE_TIME : regex::Regex = regex::Regex::new(r"(\w{2}):(\w{2})").unwrap();
}

const BLOCK_SIZE: u32 = 30;//min
const BLOCK_START: u32 = 60*9;//min

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subject
{
    pub number: u32,
    pub code: String,
    pub class_num: u8,
    pub class_name: String,
    pub prof: String,
    pub credit: u8,
    pub time_place: String,
    pub place: Vec<String>,
    pub time_tuple: [Vec<(u32, u32)>; 5],
    pub time_bit: [u64; 5]
}

impl Subject {
    pub fn new(number: u32, code: String, class_num: u8, class_name: String, prof: String, credit: u8, time_place: String) -> Self {
        let (place, time, bits) = time_and_place(time_place.clone()).unwrap();
        Subject {
            number: number,
            code: code,
            class_num: class_num,
            class_name: class_name,
            prof: prof,
            credit: credit,
            time_place: time_place,
            place: place,
            time_tuple: time,
            time_bit: bits
        }
    }

    pub fn save(subjects: &Vec<Subject>, file_name: &str) {
        let dump_string = serde_json::to_string(subjects).unwrap();
        let mut buffer = File::create(file_name).unwrap();
        buffer.write(&dump_string.into_bytes()[..]).unwrap();
    }

    pub fn load(path: &str) -> Vec<Self> {
        let file = File::open(path).unwrap();
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).unwrap()
    }

    pub fn to_array(&self) -> String {
        let mut time_string = String::from("{");
        const DAYS: [&str; 5] = ["월", "화", "수", "목","금"];
        for (idx, elem) in self.time_tuple.iter().enumerate() {
            if elem.len() == 0 {continue;}
            time_string += &format!("\"{}\":[", DAYS[idx]);
            for (s, e) in elem.iter() {
                time_string += &format!("[{},{}],", &s, &e);
            }
            if let Some(t) = time_string.pop() {
                if t != ",".chars().next().unwrap() {time_string.push(t);}
            }
            time_string += &String::from("],");
        }
        if let Some(t) = time_string.pop() {
            if t != ",".chars().next().unwrap() {time_string.push(t);}
        }
        time_string += &String::from("}");
        let mut place_string = String::from("[");
        for (idx, elem) in self.place.iter().enumerate() {
            place_string += &format!("\"{}\"", &elem.replace(" ",""));
            if idx != self.place.len()-1 {place_string += &String::from(",");} 
        }
        place_string += &String::from("]");
        format!("[{},\"{}\",{},\"{}\",\"{}\",{},{},{}]", 
        &(self.number-1), &self.code, &self.class_num, &self.class_name, &self.prof, &self.credit, place_string, time_string)
    }

    pub fn zipped_json(subjects: &Vec<Subject>) -> String {
        //let data_array = Vec::new();
        let mut s = String::from("[");
        for (i, e) in subjects.iter().enumerate() {
            s += &e.to_array();
            if i != subjects.len()-1 {s += &String::from(",")}
        }
        s += &String::from("]");
        format!("{{\"body\":{},\"head\":[\"No\",\"과목번호\",\"분반\",\"교과목명\",\"담당교수\",\"학점\",\"강의실\",\"시간\"]}}", s)
    }
}

fn time_to_num(time_str: &str) -> Result<u32, Box<dyn Error>>
{   
    let cap = RE_TIME.captures(time_str).unwrap();
    let h: u32 = cap[1].parse::<u32>()?;
    let m: u32 = cap[2].parse::<u32>()?;
    
    Ok(h*60+m)
}

fn time_to_bit(time_tuple: &(u32, u32)) -> Result<u64, Box<dyn Error>>
{
    let mut b : u64 = 0;
    for x in (time_tuple.0-BLOCK_START)/BLOCK_SIZE..(time_tuple.1-BLOCK_START)/BLOCK_SIZE
    {
        b = b + (1 << x);
    }

    Ok(b)
}

fn time_and_place(time_place_str: String) -> Result<(Vec<String>, [Vec<(u32, u32)>; 5], [u64; 5]), Box<dyn Error>>
{
    let mut place = Vec::new();
    let mut time_tuple: [Vec<(u32, u32)>; 5] = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut time_bit: [u64; 5] = [0, 0, 0, 0, 0];
    for cap in RE_DATE.captures_iter(&time_place_str[..])
    {
        place.push(cap[4].to_string());
        let t = (time_to_num(&cap[2])?, time_to_num(&cap[3])?);
        let bit = time_to_bit(&t)?;
        let i: usize;
        match &cap[1] {
            "월" => i = 0,
            "화" => i = 1,
            "수" => i = 2,
            "목" => i = 3,
            "금" => i = 4,
            _ => continue
        };
        time_tuple[i].push(t);
        if time_bit[i] | bit != time_bit[i] + bit {panic!();}
        time_bit[i] = time_bit[i] | bit;
    }
    Ok((place, time_tuple, time_bit))
}