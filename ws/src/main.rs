extern crate futures;
extern crate rsmq_async;
extern crate serde_json;
extern crate tokio;
extern crate uuid;
extern crate warp;

use futures::TryFutureExt;
use handler::Event;
use rsmq_async::{Rsmq, RsmqConnection};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;

use std::sync::Arc;

use chemix_chain::chemix::ThawBalances2;
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use warp::http::Method;
use warp::{ws::Message, Filter, Rejection};
use serde::{Serialize,Deserialize};

mod handler;
mod ws;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub topics: Vec<String>,
    pub user_address: Option<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Clone, Serialize, Debug,Deserialize)]
pub struct LastTrade2 {
    price: f64,
    amount: f64,
    height: u32,
    taker_side: String,
}

#[derive(Clone, Serialize,Deserialize)]
pub struct AddBook {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn ws_service(clients: Clients) {
    let health_route = warp::path!("health").and_then(handler::health_handler);
    let _register = warp::path("register");

    /***
    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::publish_handler);

     */

    let ws_route = warp::path("chemix")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let routes = health_route.or(ws_route).with(
        warp::cors()
            .allow_any_origin()
            //warp::cors().allow_any_origin()
            .allow_headers(vec![
                "Access-Control-Allow-Headers",
                "Access-Control-Request-Method",
                "Access-Control-Request-Headers",
                "Origin",
                "Accept",
                "X-Requested-With",
                "Content-Type",
            ])
            .allow_methods(&[
                Method::GET,
                Method::POST,
                Method::PUT,
                Method::PATCH,
                Method::DELETE,
                Method::OPTIONS,
                Method::HEAD,
            ]),
    );
    let port = match env::var_os("WS_PORT") {
        None => 7020u16,
        Some(mist_mode) => mist_mode.into_string().unwrap().parse::<u16>().unwrap(),
    };
    warp::serve(routes).run(([0, 0, 0, 0], port)).await;
}

async fn listen_msg_queue(
    mut rsmq: Rsmq,
    clients: Clients,
    queue_name: &str,
) -> Option<String> {
    let message = rsmq
        .receive_message::<String>(queue_name, None)
        .await
        .expect("cannot receive message");
    if let Some(message) = message {
        println!("receive new message {:?}", message);
        let event = Event {
            topic: queue_name.to_string(),
            user_id: None,
            message: message.message.clone(),
        };
        handler::publish_handler(event, clients).await;
        rsmq.delete_message(queue_name, &message.id).await;
        return Some(message.message);
    }
    return None;
}

#[tokio::main]
async fn main() {
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let clients_ws = clients.clone();
    let thread1 = tokio::spawn(async move {
        ws_service(clients_ws).await;
    });

    // 线程2
    let thread2 = tokio::spawn(async move {
        let mut rsmq = Rsmq::new(Default::default())
            .await
            .expect("connection failed");

        //let plus_one = |x: i32| -> i32 { x + 1 };
        //let mut rsmq_arc = Arc::new(RwLock::new(rsmq));
        loop {
            /***
            let listen_msg_queue =  |queue_name: &str| -> Option<String> async move {
                let message = rsmq
                    .receive_message::<String>(queue_name, None)
                    .await
                    .expect("cannot receive message");
                if let Some(message) = message {
                    println!("receive new message {:?}", message);
                    let event = Event {
                        topic: queue_name.to_string(),
                        user_id: None,
                        message: message.message.clone(),
                    };
                    handler::publish_handler(event, clients).await;
                    rsmq.delete_message(queue_name, &message.id).await;
                    return Some(message.message);
                }
                return None;
            };

             */

            let channel_update_book = match env::var_os("CHEMIX_MODE") {
                None => "update_book_local".to_string(),
                Some(mist_mode) => {
                    format!("update_book_{}", mist_mode.into_string().unwrap())
                }
            };

            let channel_new_trade = match env::var_os("CHEMIX_MODE") {
                None => "new_trade_local".to_string(),
                Some(mist_mode) => {
                    format!("new_trade_{}", mist_mode.into_string().unwrap())
                }
            };

            let channel_thaw_order = match env::var_os("CHEMIX_MODE") {
                None => "thaw_order_local".to_string(),
                Some(mist_mode) => {
                    format!("thaw_order_{}", mist_mode.into_string().unwrap())
                }
            };

            let message = rsmq
                .receive_message::<String>(channel_update_book.as_str(), None)
                .await
                .expect("cannot receive message");
            if let Some(message) = message {
                println!("receive new message {:?}", message);
                let markets_depth: HashMap<String,AddBook> =
                    serde_json::from_str(message.message.as_str()).unwrap();
                for market_depth in markets_depth {
                    let event = Event {
                        topic: format!("{}@depth", market_depth.0),
                        //topic: format!("human"),
                        user_id: None,
                        message: serde_json::to_string(&market_depth.1).unwrap(),
                    };
                    handler::publish_handler(event, clients.clone()).await;
                }

                rsmq.delete_message(channel_update_book.as_str(), &message.id)
                    .await;
            } else {
                tokio::time::sleep(time::Duration::from_millis(10)).await;
            }

            let message = rsmq
                .receive_message::<String>(channel_new_trade.as_str(), None)
                .await
                .expect("cannot receive message");

            if let Some(message) = message {
                //println!("receive new message {:?}", message);

                let last_trades: HashMap<String,Vec<LastTrade2>> =
                    serde_json::from_str(message.message.as_str()).unwrap();
                //遍历所有交易对逐个发送
                for last_trade in last_trades {
                    let json_str = serde_json::to_string(&last_trade.1).unwrap();
                    let event = Event {
                        topic: format!("{}@aggTrade", last_trade.0),
                        //topic: format!("human"),
                        user_id: None,
                        message: json_str,
                    };
                    handler::publish_handler(event, clients.clone()).await;
                }


                rsmq.delete_message(channel_new_trade.as_str(), &message.id)
                    .await;
            } else {
                tokio::time::sleep(time::Duration::from_millis(10)).await;
            }

            //thaw order
            let message = rsmq
                .receive_message::<String>(channel_thaw_order.as_str(), None)
                .await
                .expect("cannot receive message");
            if let Some(message) = message {
                //todo: for循环推送
                println!("receive new message {:?}", message);
                let thaw_infos: Vec<ThawBalances2> =
                    serde_json::from_str(message.message.as_str()).unwrap();
                for thaw in thaw_infos {
                    let event = Event {
                        topic: format!("{:?}@thaws",thaw.from),
                        //topic: format!("human"),
                        user_id: None,
                        message: serde_json::to_string(&thaw).unwrap(),
                    };
                    handler::publish_handler(event, clients.clone()).await;
                }

                rsmq.delete_message(channel_thaw_order.as_str(), &message.id)
                    .await;
            } else {
                tokio::time::sleep(time::Duration::from_millis(10)).await;
            }

            //let update_book = listen_msg_queue(*rsmq_arc.write().unwrap(), clients.clone(), "updateBook").await;
            //if new_trade.is_none() && update_book.is_none() {
            //   tokio::time::sleep(time::Duration::from_millis(10)).await;
            //}
        }
    });

    thread1.await.unwrap();
    thread2.await.unwrap();
}
