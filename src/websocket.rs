use futures::stream::SplitStream;
use tokio::sync::Mutex;
use futures::SinkExt;
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::{accept_async, tungstenite};
use futures::StreamExt;
use std::sync::Arc;
use serde_json::Value;

use crate::rs_println;
use crate::Args;

type Sender = Arc<Mutex<Option<futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, tungstenite::Message>>>>;
type Receiver = Arc<Mutex<Option<SplitStream<WebSocketStream<TcpStream>>>>>;

static mut GLOBAL_SENDER: Option<Sender> = None;
static mut GLOBAL_RECEIVER: Option<Receiver> = None;
static mut REPLY_HELLO: bool = false;


async fn set_sender(sender: Sender) {
  unsafe {
    GLOBAL_SENDER = Some(sender);
  }
}
async fn set_receiver(receiver: Receiver) {
  unsafe {
    GLOBAL_RECEIVER = Some(receiver);
  }
}


#[allow(static_mut_refs)]
pub async fn send_msg(msg: &str) {
  unsafe {
    if let Some(sender) = &GLOBAL_SENDER {
      let mut sender = sender.lock().await;
      if let Some(s) = sender.as_mut() {
        s.send(tungstenite::Message::Text(msg.to_string().into())).await.unwrap();
      }
    }
  }
}


#[allow(static_mut_refs)]
pub async fn send_cmd_json(func_name: &str, func_args: Value) -> Option<Value> {
  unsafe {
    let Some(sender) = &GLOBAL_SENDER else { return None };
    let mut sender = sender.lock().await;
    let Some(s) = sender.as_mut() else { return None };

    let json_str = format!(
      "json:{{\"type\": \"function\", \"value\":\"{}\", \"args\": {}}}",
      func_name, func_args
    );

    if s.send(tungstenite::Message::Text(json_str.into())).await.is_err() {
      return None;
    }

    let r = receive_response().await;
    rs_println!("Received from Python: [RESPONSE] {:?}", r);
    return r;
  }
}


#[allow(static_mut_refs)]
async fn receive_response() -> Option<Value> {
  unsafe {
    let Some(receiver) = &GLOBAL_RECEIVER else { return None };
    let mut receiver = receiver.lock().await;
    let Some(r) = receiver.as_mut() else { return None };

    let Some(Ok(msg)) = r.next().await else { return None };
    let tungstenite::Message::Text(response) = msg else { return None };

    if response.starts_with("json:") {
      return serde_json::from_str(&response[5..]).ok();
    }
    else {
      return serde_json::from_str(&response).ok();
    }
  }
}


pub async fn start(args: Args) {
  rs_println!("Starting local websocket...");
  let ip = format!("127.0.0.1:{}", args.port);
  let listener = TcpListener::bind(&ip).await.unwrap();
  rs_println!("WebSocket server running on ws://{}", ip);

  tokio::spawn(handle_connections(listener, args));
}


async fn handle_connections(listener: TcpListener, args: Args) {
  while let Ok((stream, _)) = listener.accept().await {
    let ws_stream = accept_async(stream).await.unwrap();
    let (sender, receiver) = ws_stream.split();

    let sender_arc = Arc::new(Mutex::new(Some(sender)));
    let receiver_arc = Arc::new(Mutex::new(Some(receiver)));

    set_sender(sender_arc.clone()).await;
    set_receiver(receiver_arc.clone()).await;

    while let Some(Ok(msg)) = receiver_arc.lock().await.as_mut().unwrap().next().await {
      handle_message(msg, args.clone()).await;
    }
  }
}


async fn handle_message(msg: tungstenite::protocol::Message, args: Args) {
  match msg {
    tungstenite::Message::Text(text) => {
      rs_println!("Received from Python: {}", text);

      unsafe {
        if !REPLY_HELLO {
          send_msg("[Connection test] Hello from Rust!").await;
          REPLY_HELLO = true;
        }
      }
    }
    tungstenite::Message::Binary(bytes) => {
      if args.dev {
        rs_println!("[Binary] from Python: {:?}", bytes);
      }
    }
    _ => {
      if args.dev {
        rs_println!("Received from Python: [UNKNOWN / OTHER]");
      }
    }
  }
}