# Teatable Backend
## https://jjuba.me/

Backend for Teatable service which shows possible time table from given subjects time.

## Before launch...
Please check these things.
### Check cargo.toml
#### Linux one
```toml
[package]
name = "backend-rust"
version = "0.1.0"
authors = ["hyun-wook ha <ha0146@naver.com>"]
edition = "2018"

[dependencies]
regex = "1"
csv = "1"
lazy_static = "1"
serde_json = "1.0"
serde = "1.0"
actix-cors = "0.1.0"
actix-web = { version = "1.0.5", features = ["uds"] }
redis = "*"
r2d2_redis = "0.10.1"
rand = "0.7.0"
qstring = "0.6.0"
tokio-uds = "0.2"
min-max-heap = "1.2.2"
```

#### Windows one
```toml
[package]
name = "backend-rust"
version = "0.1.0"
authors = ["hyun-wook ha <ha0146@naver.com>"]
edition = "2018"

[dependencies]
regex = "1"
csv = "1"
lazy_static = "1"
serde_json = "1.0"
serde = "1.0"
actix-cors = "0.1.0"
actix-web = "1.0"
redis = "*"
r2d2_redis = "0.10.1"
rand = "0.7.0"
qstring = "0.6.0"
min-max-heap = "1.2.2"
```

### Redis url
src/backend/DB.rs:12
```rust
lazy_static!
{
    static ref CON_POOL : r2d2::Pool<RedisConnectionManager> = {
        let manager = RedisConnectionManager::new("redis://127.0.0.1/").unwrap();
        r2d2::Pool::builder().build(manager).unwrap()
    };
}
```

### Socket or service url and cors check
#### Linux one
src/main.rs:138
```rust
fn main() {
    let server = HttpServer::new(|| {
        App::new()
        .service(web::resource("/comb").route(web::post().to(query)))
        .route("/", web::get().to(|| base(&SUB_JSON)))   
    });
    panic!("CHECK SHARE");
    let args: Vec<String> = env::args().collect();
    let default_bind = "/tmp/actix.socket".to_string();
    let bind = args.get(1).unwrap_or(&default_bind);
    print!("Service was binded to {:?}\n", bind);
    server.bind_uds(bind).unwrap().run().unwrap();
}
```

#### Windows one
src/main.rs:158
```rust
fn main() {
    let server = HttpServer::new(|| {
        App::new()
        .wrap(
            Cors::new().send_wildcard()
        )
        .service(web::resource("/comb").route(web::post().to(query)))
        .route("/", web::get().to(|| base(&SUB_JSON)))  
        .route("/test", web::get().to(test)) 
    });

    let args: Vec<String> = env::args().collect();
    let default_bind = "127.0.0.1:8088".to_string();
    let bind = args.get(1).unwrap_or(&default_bind);
    print!("Service was binded to {:?}\n", bind);
    server.bind(bind).unwrap().run().unwrap();
}
```

## Simple instruction
1. Install Cargo(https://crates.io/)
2. Build project by "Cargo build --release"
3. Run Target/Release/backend-rust(in windows, Target/Release/backend-rust.exe)



## Performance optimize

### Sortkey and using min-max heap

before: 13571ms
add sortkey: 11415ms
change to minmaxheap: 10865ms
add timeout: 11420ms