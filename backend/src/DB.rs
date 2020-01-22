use lazy_static::lazy_static;
use r2d2_redis::{r2d2, RedisConnectionManager};
use r2d2_redis::redis::Commands;

use rand::Rng;

//const SHARE_DB_NUM : u32 = 1;

lazy_static!
{
    static ref CON_POOL : r2d2::Pool<RedisConnectionManager> = {
        let manager = RedisConnectionManager::new("redis://127.0.0.1/").unwrap();
        r2d2::Pool::builder().build(manager).unwrap()
    };
}


fn make_query_string() -> String
{
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*^$@!~";
    const PASSWORD_LEN: usize = 7;
    let mut rng = rand::thread_rng();

    let password: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            // This is safe because `idx` is in range of `CHARSET`
            char::from(unsafe { *CHARSET.get_unchecked(idx) })
        })
        .collect();
    password
}

pub fn add_share(value : &Vec<u32>) -> Result<String, &str>
{
    let mut conn;

    match CON_POOL.get() {
        Ok(t) => conn = t,
        Err(_) => return Err(&"Err"),
    };
    // redis::cmd("SELECT").arg(SHARE_DB_NUM).execute(conn.deref_mut());

    let mut password = make_query_string();
    
    let mut exist : u32 = conn.exists(&password).unwrap_or(1);
    while exist == 1
    {
        password = make_query_string();
        match conn.exists(&password) {
            Ok(t) => exist = t,
            Err(_) => return Err(&"Err"),
        };
    }

    let _x : u32;
    match conn.rpush(&password, value.clone()) {
        Ok(t) => {_x = t;},
        Err(_) => return Err(&"Err"),
    };

    Ok(password)
}

pub fn get_share(key : &String) -> Option<Vec<u32>>
{
    let mut conn;
    match CON_POOL.get() {
        Ok(t) => conn = t,
        Err(_) => return None
    };

    let ans : Option<Vec<u32>> = match conn.lrange(key, 0, -1) {
        Ok(t) => {
            let v : Vec<u32> = t;
            if v.len()==0 
            {
                None
            } else {
                Some(v)
            }
        },
        Err(_) => None,
    };
    ans
}

pub fn del_share(key : &String)
{
    let mut conn = CON_POOL.get().unwrap();
    let _ans : u32 = conn.del(key).unwrap();
}