extern crate futures;
extern crate rsmq_async;
extern crate serde_json;
extern crate tokio;
extern crate uuid;
extern crate warp;

use futures::TryFutureExt;
use handler::Event;
use rsmq_async::{Rsmq, RsmqConnection, RsmqOptions};
use std::collections::HashMap;
use std::convert::Infallible;
use std::env;
use log::info;
use std::sync::Arc;

use common::queue::*;
use chemix_chain::chemix::ThawBalances2;
use serde::{Deserialize, Serialize};
//use serde_json::Value::String;
use tokio::sync::{mpsc, RwLock};
use tokio::time;
use warp::http::Method;
use warp::{ws::Message, Filter, Rejection};


mod handler;
mod ws;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;
type Queues = Arc<RwLock<Rsmq>>;


#[derive(Debug, Clone)]
pub struct Client {
    pub topics: Vec<String>,
    pub user_address: Option<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

#[derive(Clone, Serialize, Debug, Deserialize)]
pub struct LastTrade2 {
    price: f64,
    amount: f64,
    height: u32,
    taker_side: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AddBook {
    pub asks: Vec<(f64, f64)>,
    pub bids: Vec<(f64, f64)>,
}

fn with_clients(
    clients: Clients,
) -> impl Filter<Extract=(Clients, ), Error=Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn ws_service(clients: Clients) {
    let health_route = warp::path!("health").and_then(handler::health_handler);
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

async fn listen_depth(
    rsmq: Queues,
    clients: Clients,
    queue_name: &str,
) {
    loop {
        let message = rsmq.write().await
            .receive_message::<String>(queue_name, None)
            .await
            .expect("cannot receive message");
        if let Some(message) = message {
            info!("receive new message {:?}", message);
            let markets_depth: HashMap<String, AddBook> =
                serde_json::from_str(message.message.as_str()).unwrap();
            for market_depth in markets_depth {
                let event = Event {
                    topic: format!("{}@depth", market_depth.0),
                    user_id: None,
                    message: serde_json::to_string(&market_depth.1).unwrap(),
                };
                handler::publish_handler(event, clients.clone()).await;
            }

            rsmq.write().await.delete_message(queue_name, &message.id)
                .await;
        } else {
            tokio::time::sleep(time::Duration::from_millis(10)).await;
        }
    }
}


async fn listen_thaws(
    rsmq: Queues,
    clients: Clients,
    queue_name: &str,
) {
    loop {
        let message = rsmq.write().await
            .receive_message::<String>(queue_name, None)
            .await
            .expect("cannot receive message");
        if let Some(message) = message {
            info!("receive new message {:?}", message);
            let thaw_infos: Vec<ThawBalances2> =
                serde_json::from_str(message.message.as_str()).unwrap();
            for thaw in thaw_infos {
                let event = Event {
                    topic: format!("{:?}@thaws", thaw.from),
                    //topic: format!("human"),
                    user_id: None,
                    message: serde_json::to_string(&thaw).unwrap(),
                };
                handler::publish_handler(event, clients.clone()).await;
            }

            rsmq.write().await.delete_message(queue_name, &message.id)
                .await;
        } else {
            tokio::time::sleep(time::Duration::from_millis(10)).await;
        }
    }
}


async fn listen_trade(
    rsmq: Queues,
    clients: Clients,
    queue_name: &str,
) {
    loop {
        let message = rsmq.write().await
            .receive_message::<String>(queue_name, None)
            .await
            .expect("cannot receive message");

        if let Some(message) = message {
            //println!("receive new message {:?}", message);

            let last_trades: HashMap<String, Vec<LastTrade2>> =
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

            rsmq.write().await.delete_message(queue_name, &message.id)
                .await;
        } else {
            tokio::time::sleep(time::Duration::from_millis(10)).await;
        }
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();
    let clients: Clients = Arc::new(RwLock::new(HashMap::new()));
    let queue = Queue::regist(vec![QueueType::Thaws,QueueType::Depth,QueueType::Trade]).await;
    let queues: Queues = Arc::new(RwLock::new(queue));

    let clients_ws = clients.clone();
    let clients_thaws = clients.clone();
    let clients_depth = clients.clone();
    let clients_trade = clients.clone();
    let queues_thaws = queues.clone();
    let queues_depth = queues.clone();
    let queues_trade = queues.clone();


    let ws_handle = tokio::spawn(async move {
        ws_service(clients_ws).await;
    });

    //最近的解冻
    let thaws_queue = tokio::spawn(async move {
        listen_thaws(queues_thaws, clients_thaws, QueueType::Thaws.to_string().as_str()).await
    });

    //最近成交
    let agg_trade_queue = tokio::spawn(async move {
        listen_trade(queues_trade, clients_trade, QueueType::Trade.to_string().as_str()).await
    });


    //深度的增量更新
    let depth_queue = tokio::spawn(async move {
        listen_depth(queues_depth, clients_depth, QueueType::Depth.to_string().as_str()).await
    });

    let _tasks = tokio::join!(ws_handle,depth_queue,agg_trade_queue,thaws_queue);
}
