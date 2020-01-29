#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use backend;
use actix_web::{web, App, HttpResponse, HttpServer, HttpRequest};
use actix_cors::Cors;

use serde_json::{Value, json};
use serde::{Serialize, Deserialize};
use std::env;
use qstring;

mod crawler;

#[derive(Serialize, Deserialize, Debug)]
struct SearchReq {
    fix: Value,
    req: Value,
    sel: Value,
}

fn query(req : HttpRequest) -> HttpResponse
{
    let query_string = req.query_string();
    let qs = qstring::QString::from(query_string).into_pairs();

    let mut fix = "";
    let mut req = "";
    let mut sel = "";
    let mut save = "";//saving combination.
    let mut id = "";

    for (k, v) in qs.iter()
    {
        match k.as_ref() {
            "fix"  => fix  = if v != "" {v} else {"[]"},
            "req"  => req  = if v != "" {v} else {"[]"},
            "sel"  => sel  = if v != "" {v} else {"[]"},
            "save" => save = if v != "" {v} else {"[]"},
            "id" if v != "" => id = v,
            _      => {}
        };
    }

    // println!("fix : {:?}", &fix);
    // println!("req : {:?}", &req);
    // println!("sel : {:?}", &sel);
    // println!("save: {:?}", &save);
    // println!("id  : {:?}", &id);

    if (fix =="" || req =="" || sel =="") && (save == "") && (id == "")
    {
        let res = json!({"s":"f", "msg" :"비어있는 쿼리입니다."}).to_string();
        return HttpResponse::Ok().body(res);
    }


    if save != ""
    {
        let c: Result<Vec<u32>, _> = serde_json::from_str(&save[1..save.len()-1]);
        match c {
            Ok(t) => {
                if t.len()==0
                {
                    let res = json!({"s":"f", "msg" :"입력이 잘못되었습니다."}).to_string();
                    return HttpResponse::Ok().body(res);
                };
                let id;
                match backend::DB::add_share(&t) {
                    Ok(t) => id = t,
                    Err(_) => {
                        let res = json!({"s":"f", "msg" :"저장되지 못했습니다."}).to_string();
                        return HttpResponse::Ok().body(res);
                    }
                };
                let res = json!({"s":"s", "id" :id}).to_string();
                return HttpResponse::Ok().body(res);
            },
            Err(_) => {
                let res = json!({"s":"f", "msg" :"저장되지 못했습니다."}).to_string();
                return HttpResponse::Ok().body(res);
            }
        }
    }

    if id != ""
    {
        match backend::DB::get_share(&id[1..id.len()-1].to_string()) {
            Some(t) => {
                let comb = vec!(t);
                let res = json!({"s":"s", "comb" :comb}).to_string();
                return HttpResponse::Ok().body(res);
            },
            None => {
                let res = json!({"s":"f", "msg" :"가져오기에 실패했습니다."}).to_string();
                return HttpResponse::Ok().body(res);
            }
        }
    }

    let fix_subs: Vec<(&str, usize)> = serde_json::from_str(fix).unwrap_or(Vec::new());
    let mut req_subs: Vec<&str> = serde_json::from_str(req).unwrap_or(Vec::new());
    let mut sel_subs: Vec<&str> = serde_json::from_str(sel).unwrap_or(Vec::new());
    //let ans = backend::Tools::comb_sub(&SUB_MAP, &fix_subs, &mut req_subs, &mut sel_subs);
    // let res: String;
    // match ans
    // {
    //     Ok(t) =>  {
    //         match t
    //         {
    //             Some(v) => res = json!({"s":"s", "comb":v}).to_string(),
    //             None => res = json!({"s":"f", "msg" :"조합이 없습니다."}).to_string()
    //         }
    //     },
    //     Err(t) => res = json!({"s":"f", "msg" :t}).to_string()
    // }

    let res: String = json!({"s":"f", "msg" :"가져오기에 실패했습니다."}).to_string();
    HttpResponse::Ok().body(res)
}

// This function only gets compiled if the target OS is linux
#[cfg(target_os = "linux")]
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
        .service(web::resource("/comb").route(web::post().to(query)))
    });
    panic!("CHECK SHARE");
    let args: Vec<String> = env::args().collect();
    let default_bind = "/tmp/actix.socket".to_string();
    let bind = args.get(1).unwrap_or(&default_bind);
    print!("Service was binded to {:?}\n", bind);
    server.bind_uds(bind).unwrap().run().await
}

// And this function only gets compiled if the target OS is *not* linux
#[cfg(target_os = "windows")]
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();
    let default_bind = "127.0.0.1:8088".to_string();
    let bind = args.get(1).unwrap_or(&default_bind);

    let server = HttpServer::new(|| {
        App::new()
        .wrap(
            Cors::new().send_wildcard().finish()
        )
        .service(web::resource("/comb").route(web::post().to(query)))
    });
    print!("Service was binded to {:?}\n", bind);
    server.bind(bind).unwrap().run().await
}