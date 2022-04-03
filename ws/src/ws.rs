use crate::Clients;
use futures::{FutureExt, StreamExt};
use log::{error, info};
use serde::Deserialize;
use serde_json::from_str;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use warp::ws::{Message, WebSocket};

use crate::handler::{register_client, PublishRespond};
use uuid::Uuid;

#[derive(Deserialize, Debug, PartialEq)]
pub enum WSMethod {
    #[serde(rename = "UNSUBSCRIBE")]
    Unsubscribe,
    #[serde(rename = "SUBSCRIBE")]
    Subscribe,
    #[serde(rename = "GET_PROPERTY")]
    GetProperty,
    #[serde(rename = "PING")]
    Ping,
    #[serde(rename = "PONG")]
    Pong,
}

/***{
"method": "SUBSCRIBE",
"params": {
"hash":"0x90a97d253608B2090326097a44eA289d172c30Ec",
"channel": "BTC-USDT@aggTrade",

}
}
***/

#[derive(Deserialize, Debug, PartialEq)]
pub struct Params {
    hash: String,
    channel: Vec<String>,
}
#[derive(Deserialize, Debug)]
pub struct TopicsRequest2 {
    method: WSMethod,
    params: Params,
}

//{"method": "PING","params":{"channel":[],"hash":""}}
//{"method": "SUBSCRIBE","params":{"channel":["BTC-USDT@kline_1d","BTC-USDT@aggTrade","BTC-USDT@depth"],"hash":""}}

//{"method": "SUBSCRIBE", "params": ["ethbusd@kline_1d","ethbusd@aggTrade","ethbusd@depth"]}
//["miniTicker@arr@3000ms", "ethbusd@aggTrade", "ethbusd@kline_1d", "ethbusd@depth"]
//{"method":"UNSUBSCRIBE","params":["!miniTicker@arr@3000ms","ethbusd@aggTrade","ethbusd@kline_1d","ethbusd@depth

pub async fn client_connection(
    ws: WebSocket,
    clients: Clients,
    //mut client: Client,
) {
    let (client_ws_sender, mut client_ws_rcv) = ws.split();
    let (client_sender, client_rcv) = mpsc::unbounded_channel();
    let client_rcv = UnboundedReceiverStream::new(client_rcv); // <-- this

    //insert new client
    let id = Uuid::new_v4().simple().to_string();
    register_client(id.clone(), client_sender, clients.clone()).await;

    tokio::task::spawn(client_rcv.forward(client_ws_sender).map(|result| {
        if let Err(e) = result {
            error!("error sending websocket msg: {}", e);
        }
    }));

    while let Some(result) = client_ws_rcv.next().await {
        let msg = match result {
            Ok(msg) => msg,
            Err(e) => {
                error!("error receiving ws message : {}", e);
                break;
            }
        };
        client_msg(&id, msg, &clients).await;
    }

    clients.write().await.remove(&id);
    info!("{} disconnected", id);
}

async fn client_msg(id: &str, msg: Message, clients: &Clients) {
    info!("received message from {}: {:?}", id, msg);
    let message = match msg.to_str() {
        Ok(v) => v,
        Err(_) => return,
    };

    let topics_req: TopicsRequest2 = match from_str(&message) {
        Ok(v) => v,
        Err(e) => {
            error!("error while parsing message to topics request: {}", e);
            return;
        }
    };

    let mut locked = clients.write().await;
    if let Some(v) = locked.get_mut(id) {
        info!("new subcribe topics {:?}", topics_req.params.channel);
        match topics_req.method {
            WSMethod::Subscribe => {
                for item in topics_req.params.channel {
                    if !v.topics.contains(&item) {
                        v.topics.push(item)
                    }
                }

                //hash保留字段暂时没用,
                match topics_req.params.hash.as_str() {
                    "" => {
                        v.user_address = None;
                    }
                    _ => {
                        v.user_address = Some(topics_req.params.hash.to_lowercase());
                    }
                }

                if let Some(_sender) = &v.sender {
                    //todo: 可能要推全量数据
                }
            }
            WSMethod::Unsubscribe => {
                for param in topics_req.params.channel {
                    v.topics.retain(|x| *x != param);
                }
            }
            WSMethod::GetProperty => {
                todo!()
            }
            WSMethod::Ping => {
                let respond = PublishRespond {
                    method: "PONG".to_string(),
                    channel: "".to_string(),
                    data: "".to_string(),
                };
                let respond_str = serde_json::to_string(&respond).unwrap();
                if let Some(sender) = &v.sender {
                    let _ = sender.send(Ok(Message::text(&respond_str)));
                }
            }
            WSMethod::Pong => {}
        }
    }
}
