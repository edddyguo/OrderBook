extern crate tokio;

use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use warp::{ws::Message, Filter, Rejection};
use tokio::time;
use handler::Event;
use rsmq_async::{Rsmq, RsmqConnection};

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

fn with_clients(clients: Clients) -> impl Filter<Extract = (Clients,), Error = Infallible> + Clone {
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
        .with(warp::cors().allow_any_origin());

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
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
        loop {
            let message = rsmq
                .receive_message::<String>("myqueue", None)
                .await
                .expect("cannot receive message");
            if let Some(message) = message {
                println!("receive new message {:?}",message);
                let event = Event {
                    topic: "human".to_string(),
                    user_id: None,
                    message: message.message,
                };
                handler::publish_handler(event,clients.clone()).await;
                rsmq.delete_message("myqueue", &message.id).await;
            }else {
                //tokio::time::sleep(time::Duration::from_secs(1)).await;
                tokio::time::sleep(time::Duration::from_millis(10)).await;

                println!("have no  new message");
                continue;
            }
            //tokio::time::sleep(time::Duration::from_secs(5)).await;
            /***
            let event = Event {
                topic: "human".to_string(),
                user_id: None,
                message: "test1".to_string(),
            };
            handler::publish_handler(event,clients.clone()).await;

             */
        }

    });

    thread1.await.unwrap();
    thread2.await.unwrap();

}


