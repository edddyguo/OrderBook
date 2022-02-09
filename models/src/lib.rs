pub mod api;
pub mod chain;
pub mod engine;

#[macro_use]
extern crate jsonrpc_client_core;
extern crate jsonrpc_client_http;

use postgres::{Client, Error, NoTls};
use std::env;
use std::fs::OpenOptions;
use std::sync::Mutex;

#[macro_use]
extern crate lazy_static;
extern crate chrono;
extern crate postgres;

use chrono::prelude::*;
use chrono::Local;
use std::ptr::null;
use std::time::Instant;

lazy_static! {
    static ref CLIENTDB: Mutex<postgres::Client> = Mutex::new({ connetDB().unwrap() });
}

pub fn restartDB() -> bool {
    let now = Local::now();
    println!("restart postgresql {:?}", now);
    // let client =  connetDB();
    if let Some(client) = connetDB() {
        *crate::CLIENTDB.lock().unwrap() = client;
        return true;
    }
    false
}

fn connetDB() -> Option<postgres::Client> {
    let mut client;
    let mut dbname = "chemix".to_string();
    if let Some(mist_mode) = env::var_os("MIST_MODE") {
        dbname = mist_mode.into_string().unwrap();
    } else {
        eprintln!("have no MIST_MODE env");
    }

    let url = format!(
        "host=localhost user=postgres port=5432 password=postgres dbname={}",
        dbname
    );

    match Client::connect(&url, NoTls) {
        Ok(tmp) => {
            client = tmp;
            eprintln!("connect postgresql successfully");
        }
        Err(error) => {
            eprintln!("connect postgresql failed,{:?}", error);
            return None;
        }
    };
    Some(client)
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
