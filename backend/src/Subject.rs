use std::error::Error;
use std::fs::File;
use std::collections::HashMap;
use lazy_static::lazy_static;
use regex;
use serde_json::json;

lazy_static! {
static ref RE_DATE : regex::Regex = regex::Regex::new(r"([월화수목금토일])+(\w{2}:\w{2})-(\w{2}:\w{2})\((\w{2}\s?-\s?\w*)\)").unwrap();
static ref RE_TIME : regex::Regex = regex::Regex::new(r"(\w{2}):(\w{2})").unwrap();
}

const BLOCK_SIZE: u32 = 30;//min
const BLOCK_START: u32 = 60*9;//min

#[derive(Debug)]
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
        let (place, time, bits) = time_and_place(&time_place).unwrap();
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

fn time_and_place(time_place_str: &String) -> Result<(Vec<String>, [Vec<(u32, u32)>; 5], [u64; 5]), Box<dyn Error>>
{
    let mut place = Vec::new();
    let mut time_tuple: [Vec<(u32, u32)>; 5] = [Vec::new(), Vec::new(), Vec::new(), Vec::new(), Vec::new()];
    let mut time_bit: [u64; 5] = [0, 0, 0, 0, 0];
    for cap in RE_DATE.captures_iter(time_place_str)
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


#[deprecated(
    since = "2020",
    note = "Do not use any csv. Use reqeust"
)]
pub fn read_csv(file_path : &str) -> Result<HashMap<String, Vec<Subject>>, Box<dyn Error>>
{
    let file = File::open(file_path)?;
    let mut rdr = csv::ReaderBuilder::new().has_headers(false).from_reader(file);

    let mut subjects = HashMap::new();
    for result in rdr.records()
    {
        let record = result?;
        let code = record[3].to_string();

        let time_place = record[16].to_string().replace(" ", "");
        let (place, time_tuple, time_bit) = time_and_place(&time_place)?;

        let sub = Subject { 
            number: record[0].parse::<u32>()?-1, 
            code: code.clone(), 
            class_num: record[4].parse::<u8>()?,
            class_name: record[5].to_string(),
            prof: record[6].to_string(),
            credit: record[13].parse::<f32>()? as u8,
            time_place: time_place,
            place: place,
            time_tuple: time_tuple,
            time_bit: time_bit};
        subjects.entry(code).or_insert_with(Vec::new).push(sub);
    }
    Ok(subjects)
}

#[deprecated(
    since = "2020",
    note = "Do not use this to generate json."
)]
pub fn get_json(subdata: &HashMap<String, Vec<Subject>>) -> String
{
    let head = vec!["No", "과목번호", "분반", "교과목명", "담당교수", "학점", "강의실", "시간"];
    let mut data = Vec::new();
    for (_, v) in subdata.iter()
    {
        for s in v.iter()
        {
            let mut time : serde_json::Map<String, serde_json::Value> = serde_json::Map::new();
            for (d, t) in s.time_tuple.iter().enumerate()
            {
                if t.len() ==0 
                {
                    continue;
                }
                let key: String;
                match d {
                    0 => key = "월".to_string(),
                    1 => key = "화".to_string(),
                    2 => key = "수".to_string(),
                    3 => key = "목".to_string(),
                    4 => key = "금".to_string(),
                    _ => panic!(),
                }
                time.insert(key, json!(t));
            }

            data.push(vec![
                json!(s.number),
                json!(s.code.clone()),
                json!(s.class_num),
                json!(s.class_name.clone()),
                json!(s.prof.clone()),
                json!(s.credit),
                json!(s.place.clone()),
                json!(time)
            ]);
        }
    }

    data.sort_unstable_by_key(|x| x[0].as_i64().unwrap());

    json!({"head":json!(head),"body": data}).to_string()
}