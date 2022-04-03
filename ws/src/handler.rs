use crate::{ws, Client, Clients, Result};
use log::info;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use warp::{http::StatusCode, ws::Message, Reply};

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
    pub(crate) message: String,
}

pub async fn publish_handler(body: Event, clients: Clients) -> Result<impl Reply> {
    let respond = PublishRespond {
        channel: body.topic.clone(),
        method: "SUBSCRIBE".to_string(),
        data: body.message.clone(),
    };
    let respond_str = serde_json::to_string(&respond).unwrap();
    info!("respond_str {:?}", respond_str);
    clients
        .read()
        .await
        .iter()
        .filter(|(id, _)| match body.user_id {
            Some(v) => **id == v.to_string(),
            None => true,
        })
        .filter(|(_, client)| {
            let lowercase_client_topic = client
                .topics
                .iter()
                .map(|x| x.to_lowercase())
                .collect::<Vec<String>>();
            lowercase_client_topic.contains(&body.topic.to_lowercase())
        })
        .for_each(|(_, client)| {
            if let Some(sender) = &client.sender {
                let _res = sender.send(Ok(Message::text(respond_str.clone()))).unwrap();
            }
        });

    Ok(StatusCode::OK)
}

pub async fn register_client(
    id: String,
    client_sender: mpsc::UnboundedSender<std::result::Result<Message, warp::Error>>,
    clients: Clients,
) {
    clients.write().await.insert(
        id,
        Client {
            topics: vec![String::from("")],
            user_address: None,
            sender: Some(client_sender),
        },
    );
}

pub async fn ws_handler(ws: warp::ws::Ws, clients: Clients) -> Result<impl Reply> {
    Ok(ws.on_upgrade(move |socket| ws::client_connection(socket, clients)))
}

pub async fn health_handler() -> Result<impl Reply> {
    Ok(StatusCode::OK)
}
