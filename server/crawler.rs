use actix_web::client::{Connector, ClientBuilder};
use actix_web::http::header::{ContentType};
use openssl::ssl::{SslConnector, SslVerifyMode, SslMethod};
use serde::{Serialize, Deserialize};
use backend::Subject::*;
use std::io::prelude::*;
use std::fs::File;

const SUBJECT_URL: &'static str = "https://welcome.dgist.ac.kr/ucs/ucsqProfRespSbjtInq/list.do;";

enum Semister {
    Spring,
    Summer,
    Fall,
    Winter
}

enum Organization {
    UnderGraduate,
    Graduate,
}

pub struct SubjectQuery {
    year: u32,
    semister: Semister,
    organization: Organization,
}

#[derive(Serialize, Deserialize)]
pub struct SubjectResponse {
    page: String,
    total: u32,
    records: String,
    user: Vec<ResponseUnit>
}

impl SubjectResponse {
    pub fn to_subject_vector(self) -> Vec<Subject> {
        self.user.into_iter().map(|x| x.to_subject()).collect()
    }
    pub fn dump(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    pub fn from_file(file_name: String) -> Self {
        let mut file = File::open(file_name).unwrap();
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer).unwrap();
        let v: SubjectResponse = serde_json::from_slice(&buffer[..]).unwrap();
        v
    }
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct ResponseUnit {
    ORGN_CLSF_NM: String,
    ASGN_SUST_NM: String,
    SBJT_NM: String,
    CPTN_NM: String,
    SBJT_DETA_NM: String,
    LT_PRAC_NM: String,
    SBJT_FELD_NM: String,
    FELD_DETA_NM: String,
    MRKS_APPR_MTHD_NM: String,
    MRKS_GV_MTHD_NM: String,
    PNT: String,
    THEO_TMCNT: String,
    PRAC_TMCNT: String,
    PROF_NM: String,
    PROF_NO: String,
    OPEN_CORS_NM: String,
    SBJT_NO: String,
    CLSS_NO: String,
    ORGN_CLSF_DCD: String,
    SHYY: String,
    SHTM_DCD: String,
    ASGN_SUST_CD: String,
    ASGN_CORS_DCD: String,
    TLSN_TIME: Option<String>,
    LECPLN_CNT: u32,
    RNUM: u32
}

impl ResponseUnit {
    pub fn to_subject(self) -> Subject {
        Subject::new(
            self.RNUM,
            self.SBJT_NO,
            self.CLSS_NO.parse::<u8>().unwrap(),
            self.SBJT_NM,
            self.PROF_NM,
            self.PNT.parse::<f32>().unwrap() as u8,
            self.TLSN_TIME.unwrap_or("".to_string())
        )
    }
}

impl SubjectQuery {
    pub fn new(year: u32) -> SubjectQuery {
        SubjectQuery {
            year: year,
            semister: Semister::Spring,
            organization: Organization::UnderGraduate,
        }
    }

    pub fn spring(&mut self) -> &mut SubjectQuery {
        self.semister = Semister::Spring;
        self
    }

    pub fn summer(&mut self) -> &mut SubjectQuery {
        self.semister = Semister::Summer;
        self
    }

    pub fn fall(&mut self) -> &mut SubjectQuery {
        self.semister = Semister::Fall;
        self
    }

    pub fn winter(&mut self) -> &mut SubjectQuery {
        self.semister = Semister::Winter;
        self
    }

    pub fn undergraduate(&mut self) -> &mut SubjectQuery {
        self.organization = Organization::UnderGraduate;
        self
    }

    pub fn graduate(&mut self) -> &mut SubjectQuery {
        self.organization = Organization::Graduate;
        self
    }

    pub async fn send(&mut self) -> Result<SubjectResponse, String>{
        let mut ssl_conn_builder = match SslConnector::builder(SslMethod::tls()) {
            Ok(t) => t,
            Err(_) => return Err(String::from("Fail to make ssl connector"))
        };

        ssl_conn_builder.set_verify(SslVerifyMode::NONE);
        let ssl_conn = ssl_conn_builder.build();
        let conn = Connector::new().ssl(ssl_conn).finish();

        let client = ClientBuilder::new().connector(conn).finish();
        let mut a = match client.get(SUBJECT_URL)
        .set(ContentType::form_url_encoded())
        .send_body(format!("pageSize=99999&searchLang=ko&searchOrgnClsfDcd={}&selectYearTerm={:4}{}", 
        match self.organization {
            Organization::Graduate => "CMN12.02",
            Organization::UnderGraduate => "CMN12.03",
        }, 
        self.year,
        match self.semister {
            Semister::Spring => "CMN17.10",
            Semister::Summer => "CMN17.11",
            Semister::Fall => "CMN17.20",
            Semister::Winter => "CMN17.21",
        }
        )).await {
            Ok(t) => t,
            Err(t) => return Err(format!("Fail to get response Err: {}", &t))
        };
        match a.body().await {
            Ok(t) => {
                let req: SubjectResponse = match serde_json::from_slice(&t) {
                    Ok(t) => t,
                    Err(_) => return Err(String::from("Fail to parse"))
                };
                Ok(req)
            },
            Err(_) => Err(String::from("Fail to get body of request"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[actix_rt::test]
    async fn test_request() {
        let a = SubjectQuery::new(2019).fall().undergraduate().send().await.unwrap();
        println!("{:?} {:?}", &a.user[0].SBJT_NM, &a.user[0].TLSN_TIME);
    }

    // use backend::*;
    // use std::collections::HashMap;
    // #[actix_rt::test]
    // async fn test_combination() {
    //     let a = SubjectQuery::new(2019).fall().undergraduate().send().await.unwrap();
    //     let dump_string = a.dump();
    //     let mut buffer = File::create("foo.txt").unwrap();
    //     buffer.write(&dump_string.into_bytes()[..]).unwrap();


    //     let subject_vec = a.to_subject_vector();
    //     let combinator = Tools::SubjectCombinator::new(&subject_vec);
    //     let fix_subs = vec![("SE324a".to_string(), 0), ("SE334a".to_string(), 0), ("SE380".to_string(), 0), ("HL303".to_string(), 31)];
    //     let mut req_subs = vec!["HL203".to_string(), "HL204".to_string(), "HL305".to_string()];
    //     let mut sel_subs = vec!["HL320".to_string()];
    //     let ans = combinator.comb_sub(&fix_subs, &mut req_subs, &mut sel_subs);
    //     println!("{:?}", ans.unwrap().unwrap().len());

    //     let mut subjects = HashMap::new();
    //     for sub in subject_vec.into_iter() {
    //         subjects.entry(sub.code.clone()).or_insert_with(Vec::new).push(sub);
    //     }

    //     let ans2 = Tools::comb_sub(&subjects, &fix_subs, &mut req_subs, &mut sel_subs);
    //     println!("{:?}", ans2.unwrap().unwrap().len());
    // }
}