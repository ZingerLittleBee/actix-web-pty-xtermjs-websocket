mod pty_manager;

use std::fmt::Display;
use crate::pty_manager::PtyManager;

use actix::{Actor, AsyncContext, Message, StreamHandler};
use actix_web::{web, Error, HttpRequest, HttpResponse, HttpServer, App};
use actix_web_actors::ws;
use std::sync::{Arc, Mutex};
use serde::__private::from_utf8_lossy;

#[derive(Message, Debug, Eq, PartialEq)]
#[rtype(result = "()")]
pub enum PtyMessage {
    Buffer(Vec<u8>),
}

impl Display for PtyMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PtyMessage::Buffer(data) => {
                write!(f, "Buffer({:?})", from_utf8_lossy(data))
            }
        }
    }
}

struct MyWs {
    pty: Arc<Mutex<PtyManager>>,
}

impl MyWs {
    fn new() -> Self {
        Self {
            pty: Arc::new(Mutex::new(PtyManager::new())),
        }
    }
}

impl actix::Handler<PtyMessage> for MyWs {
    type Result = ();

    fn handle(&mut self, msg: PtyMessage, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            PtyMessage::Buffer(data) => {
                ctx.binary(data);
            }
        }
    }
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        ctx.text("  _____                            ____\r\n");
        ctx.text(" / ____|                          |  _ \\\r\n");
        ctx.text("| (___   ___ _ ____   _____ _ __  | |_) | ___  ___\r\n");
        ctx.text(" \\___ \\ / _ \\ '__\\ \\ / / _ \\ '__| |  _ < / _ \\/ _ \\\r\n");
        ctx.text(" ____) |  __/ |   \\ V /  __/ |    | |_) |  __/  __/\r\n");
        ctx.text("|_____/ \\___|_|    \\_/ \\___|_|    |____/ \\___|\\___|\r\n");
        ctx.text(format!("Version: {}\r\n", env!("CARGO_PKG_VERSION")));

        ctx.text("\r\n");

        ctx.text("Website: https://serverbee.app\r\n");
        ctx.text("Documentation: https://docs.serverbee.app\r\n");
        ctx.text("Enjoy your journey!\r\n");

        let rx = self.pty.lock().unwrap().start();

        let addr = ctx.address();

        std::thread::spawn(move || {
            loop {
                match rx.recv() {
                    Ok(msg) => {
                        addr.do_send(msg);
                    }
                    Err(e) => {
                        println!("recv error: {}", e);
                    }
                }
            }
        });
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                self.pty.lock().unwrap().write_to_pty(&text).unwrap();
            }
            Ok(ws::Message::Binary(bin)) => {
                ctx.binary(bin)
            }
            _ => (),
        }
    }
}
async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    let resp = ws::start(MyWs::new(), &req, stream);
    resp
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().route("/ws", web::get().to(index)))
        .bind(("0.0.0.0", 8080))?
        .run()
        .await
}
