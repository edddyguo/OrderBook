use crate::{ws, Client, Clients, Result};
use serde::{Deserialize, Serialize};

use warp::{http::StatusCode, ws::Message, Reply};
use chemix_chain::chemix::ThawBalances;

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    user_id: usize,
}

#[derive(Serialize, Debug)]
pub struct PublishRespond {
    pub channel: String,
    pub method: String,
    pub data: String,
}

#[derive(Deserialize, Debug)]
pub struct Event {
    pub(crate) topic: String,
    pub(crate) user_id: Option<usize>,
    pub(crate) user_address: Option<String>,
    pub(crate) message: String,
}


pub async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply> {
    let respond = PublishRespond {
        channel: body.topic.clone(),
        method: "SUBSCRIBE".to_string(),
        data: body.message.clone(),
    };
    let respond_str = serde_json::to_string(&respond).unwrap();
    println!("-==========={:?}", respond_str);
    clients
        .read()
        .await
        .iter()
        .filter(|(id, _)| match body.user_id {
            Some(v) => **id == v.to_string(),
            None => true,
        })
        .filter(|(_, client)| client.topics.contains(&body.topic))
        .filter(|(_, client)| {
            match body.user_address.clone() {
                None => {
                    true
                },
                Some(addr) => {
                    println!("user_addr={},body_addr={},eq={}", client.user_address.as_ref().unwrap().to_string(),addr,client.user_address.as_ref().unwrap().to_string() == addr);
                    client.user_address.as_ref().unwrap().to_string() == addr
                }
            }
        })
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                println!("-==========={:?}", respond_str);
                let _ = sender.send(Ok(Message::text(respond_str.clone())));
                println!("+++++++{:?}", respond_str);
            }
        });

    Ok(StatusCode::OK)
}

async fn register_client(id: String, _user_id: usize, clients: Clients) {
    clients.write().await.insert(
        id,
        Client {
            topics: vec![String::from("cats")],
            user_address: None,
            sender: None,
        },
    );
}

pub async fn ws_handler(ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients)))
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
