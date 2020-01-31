#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

use backend;
use actix_web::{web, App, HttpResponse, HttpServer};
use actix_cors::Cors;

use serde_json::json;
use serde::Deserialize;
use std::env;
use std::path::Path;

use r2d2_redis::{r2d2, RedisConnectionManager};

mod crawler;

#[derive(Deserialize)]
struct DBJson {
    id: Option<String>,
    save: Option<Vec<u32>>
}

async fn db_access(json: web::Json<DBJson>, conn_pool: web::Data<r2d2::Pool<RedisConnectionManager>>) -> HttpResponse {
    match &json.id {
        Some(t) => {
            match backend::DB::get_share(conn_pool.as_ref(), &t) {
                Some(comb) => {
                    let comb = vec!(comb);
                    let res = json!({"s":"s", "comb" :comb}).to_string();
                    return HttpResponse::Ok().body(res);
                },
                None => {
                    let res = json!({"s":"f", "msg" :"가져오기에 실패했습니다."}).to_string();
                    return HttpResponse::Ok().body(res);
                }
            }
        },
        None => {
            match &json.save {
                Some(v) => {
                    match backend::DB::add_share(conn_pool.as_ref(), &v) {
                        Ok(id) => {
                            let res = json!({"s":"s", "id" :id}).to_string();
                            return HttpResponse::Ok().body(res);
                        },
                        Err(_) => {
                            let res = json!({"s":"f", "msg" :"저장되지 못했습니다."}).to_string();
                            return HttpResponse::Ok().body(res);
                        }
                    }
                },
                None => {
                    let res = json!({"s":"f", "msg" :"부적합한 쿼리입니다."}).to_string();
                    return HttpResponse::Ok().body(res);
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct CombinationJson {
    fix: Vec<(String, usize)>,
    req: Vec<String>,
    sel: Vec<String>,
}

async fn combination(json: web::Json<CombinationJson>, combinator: web::Data<backend::Tools::SubjectCombinator>) -> HttpResponse
{
    let CombinationJson{ fix, mut req, mut sel } = json.into_inner();
    let ans = combinator.combinate_subjects(&fix, &mut req, &mut sel);

    let res: String = match ans {
        Ok(value) => match value {
            Some(arr) => {
                let comb: Vec<&Vec<usize>> = arr.iter().map(
                        |x| x.as_ref()
                    ).collect();
                json!({"s":"t", "comb":comb}).to_string()
            },
            None => json!({"s":"f", "msg":"조합이 없습니다."}).to_string(),
        },
        Err(_) => json!({"s":"f", "msg": "조합에 실패했습니다."}).to_string()
    };
    HttpResponse::Ok().body(res)
}

async fn data(data: web::Data<String>) -> HttpResponse
{
    HttpResponse::Ok().body(data.as_ref())
}

// This function only gets compiled if the target OS is linux
#[cfg(target_os = "linux")]
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use backend::Subject::Subject as Subject;

    let args: Vec<String> = env::args().collect();
    let default_bind = "/tmp/actix.socket".to_string();
    let bind = args.get(1).unwrap_or(&default_bind);

    let subject_vec: Vec<Subject>;

    if Path::new("data.json").exists() {
        subject_vec = Subject::load("data.json");
    }
    else {
        let a = crawler::SubjectQuery::new(2019).fall().undergraduate().send().await.unwrap();
        subject_vec = a.to_subject_vector();
        backend::Subject::Subject::save(&subject_vec, "data.json");
    }

    let conn_pool = r2d2::Pool::builder().build(
        RedisConnectionManager::new("redis://127.0.0.1/").unwrap()
    ).unwrap();

    let combinator = backend::Tools::SubjectCombinator::new(subject_vec.clone());

    let server = HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::new().send_wildcard().finish()
        )
        .data(combinator.clone())
        .data(conn_pool.clone())
        .service(web::resource("/comb").route(web::post().to(combination)))
        .service(web::resource("/share").route(web::post().to(db_access)))
        //.service(web::resource("/").route(web::post().to(index)))
    });
    print!("Service was binded to {:?}\n", bind);
    server.bind_uds(bind).unwrap().run().await
}

// And this function only gets compiled if the target OS is *not* linux
#[cfg(target_os = "windows")]
#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    use backend::Subject::Subject as Subject;

    let args: Vec<String> = env::args().collect();
    let default_bind = "127.0.0.1:8088".to_string();
    let bind = args.get(1).unwrap_or(&default_bind);

    let subject_vec: Vec<Subject>;

    if Path::new("data.json").exists() {
        subject_vec = Subject::load("data.json");
    }
    else {
        let a = crawler::SubjectQuery::new(2019).fall().undergraduate().send().await.unwrap();
        subject_vec = a.to_subject_vector();
        backend::Subject::Subject::save(&subject_vec, "data.json");
    }

    let conn_pool = r2d2::Pool::builder().build(
        RedisConnectionManager::new("redis://127.0.0.1/").unwrap()
    ).unwrap();

    let combinator = backend::Tools::SubjectCombinator::new(subject_vec.clone());
    let data_string = Subject::zipped_json(&subject_vec);

    let server = HttpServer::new(move || {
        App::new()
        .wrap(
            Cors::new().send_wildcard().finish()
        )
        .data(combinator.clone())
        .data(conn_pool.clone())
        .data(data_string.clone())
        .service(web::resource("/comb").route(web::post().to(combination)))
        .service(web::resource("/share").route(web::post().to(db_access)))
        .service(web::resource("/data").route(web::post().to(data)))
    });
    print!("Service was binded to {:?}\n", bind);
    server.bind(bind).unwrap().run().await
}