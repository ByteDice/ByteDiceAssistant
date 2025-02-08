use tokio::sync::Mutex;
use futures::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async, tungstenite};
use futures::StreamExt;
use std::sync::Arc;

use crate::rs_println;
use crate::Args;

type Sender = Arc<Mutex<Option<futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, tungstenite::Message>>>>;

static mut GLOBAL_SENDER: Option<Sender> = None;
static mut REPLY_HELLO: bool = false;


async fn set_sender(sender: Sender) {
  unsafe {
    GLOBAL_SENDER = Some(sender);
  }
}


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
    let (sender, mut receiver) = ws_stream.split();

    let sender_arc = Arc::new(Mutex::new(Some(sender)));
    set_sender(sender_arc.clone()).await;

    while let Some(Ok(msg)) = receiver.next().await {
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