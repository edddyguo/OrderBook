extern crate tokio;
extern crate rsmq_async;
extern crate futures;
extern crate serde_json;
extern crate warp;

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};
use tokio::time;
use handler::Event;
use rsmq_async::{Rsmq, RsmqConnection};
use std::ops::Deref;
use std::rc::Rc;
use futures::TryFutureExt;
use tokio::runtime::Runtime;
use std::sync::RwLock as StdRwlock;
use warp::http::Method;


mod handler;
mod ws;

type Result<T> = std::result::Result<T, Rejection>;
type Clients = Arc<RwLock<HashMap<String, Client>>>;

#[derive(Debug, Clone)]
pub struct Client {
    pub user_id: usize,
    pub topics: Vec<String>,
    pub sender: Option<mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>>,
}

fn with_clients(clients: Clients) -> impl Filter<Extract=(Clients, ), Error=Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

async fn ws_service(clients: Clients) {
    let health_route = warp::path!("health").and_then(handler::health_handler);
    let register = warp::path("register");
    let register_routes = register
        .and(warp::post())
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::register_handler)
        .or(register
            .and(warp::delete())
            .and(warp::path::param())
            .and(with_clients(clients.clone()))
            .and_then(handler::unregister_handler));

    /***
    let publish = warp::path!("publish")
        .and(warp::body::json())
        .and(with_clients(clients.clone()))
        .and_then(handler::publish_handler);

     */

    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(warp::path::param())
        .and(with_clients(clients.clone()))
        .and_then(handler::ws_handler);

    let routes = health_route
        .or(register_routes)
        .or(ws_route)
        .with(warp::cors().allow_any_origin()
                //warp::cors().allow_any_origin()
            .allow_headers(vec!["Access-Control-Allow-Headers", "Access-Control-Request-Method", "Access-Control-Request-Headers", "Origin", "Accept", "X-Requested-With", "Content-Type"])
                  .allow_methods(&[Method::GET, Method::POST, Method::PUT, Method::PATCH, Method::DELETE, Method::OPTIONS, Method::HEAD])
    );

    warp::serve(routes).run(([0, 0, 0, 0], 7020)).await;
}

async fn listen_msg_queue(mut rsmq: Rsmq, clients: Clients, queue_name: &str) -> Option<String> {
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

            let message = rsmq
                .receive_message::<String>("updateBook", None)
                .await
                .expect("cannot receive message");
            if let Some(message) = message {
                println!("receive new message {:?}", message);
                let event = Event {
                    topic: format!("{}@depth","BTC-USDT"),
                    //topic: format!("human"),
                    user_id: None,
                    message: message.message.clone(),
                };
                handler::publish_handler(event, clients.clone()).await;
                rsmq.delete_message("updateBook", &message.id).await;
            }else {
                tokio::time::sleep(time::Duration::from_millis(10)).await;
            }


            let message = rsmq
                .receive_message::<String>("newTrade", None)
                .await
                .expect("cannot receive message");
            if let Some(message) = message {
                println!("receive new message {:?}", message);
                let event = Event {
                    topic: format!("{}@aggTrade","BTC-USDT"),
                    //topic: format!("human"),
                    user_id: None,
                    message: message.message.clone(),
                };
                handler::publish_handler(event, clients.clone()).await;
                rsmq.delete_message("newTrade", &message.id).await;
            }else {
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


