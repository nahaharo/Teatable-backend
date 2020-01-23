use http::{Request, Response};
use std::fmt;

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

    pub fn underGraduate(&mut self) -> &mut SubjectQuery {
        self.organization = Organization::UnderGraduate;
        self
    }

    pub fn graduate(&mut self) -> &mut SubjectQuery {
        self.organization = Organization::Graduate;
        self
    }

    pub fn make_request(&mut self) -> Request<String> {
        Request::post(SUBJECT_URL)
        .body(String::from("pageSize:99999")).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_request() {
        let a = SubjectQuery::new(2019).spring().underGraduate().make_request();
        println!("{:?}", &a);
    }
}