use serde::{Deserialize, Serialize};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub mod server;

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientFailed {
    server_command: String,
    current_position: Point,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientOnConnect {
    pub client_name: String,
    pub message: String,
    pub current_position: Point,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ClientCommandResponse {
    pub message: String,
    pub current_position: Point,
    // Here we could add an a set of error codes
}

// These are the messages the client will send to the server
#[derive(Deserialize, Serialize, Debug)]
pub enum ClientMessage {
    ClientOnConnect(ClientOnConnect),
    ClientCommandResponse(ClientCommandResponse),
    Failed(ClientFailed),
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerOnConnect {
    pub client_name: String,
    pub message: String,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerCommand {
    pub success_message: String,
    pub message_id: String,
}

// These are the messages the server will send to the client
#[derive(Deserialize, Serialize, Debug)]
pub enum ServerMessage {
    OnConnect(ServerOnConnect),
    ServerMoveCommand(Point),
    ServerFailed(String),
}

type FramedStream = FramedRead<OwnedReadHalf, LengthDelimitedCodec>;
type FramedSink = FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>;
type ClientStream = Framed<FramedStream, ClientMessage, (), Json<ClientMessage, ()>>;
type ServerSink = Framed<FramedSink, (), ServerMessage, Json<(), ServerMessage>>;

// This is provides some reasonable ergonomics around working with tcp.
// https://github.com/carllerche/tokio-serde/blob/master/examples/server.rs
fn wrap_stream(stream: TcpStream) -> (ClientStream, ServerSink) {
    // here we first split the stream into read and write this will allow us to work with them each separately
    let (read, write) = stream.into_split();

    // here were wrapping them is a framed and length delimited codec.
    // this let's us not have to worry about buffering and provides deserialization using serde
    let stream = FramedStream::new(read, LengthDelimitedCodec::new());
    let sink = FramedSink::new(write, LengthDelimitedCodec::new());
    (
        ClientStream::new(stream, Json::default()),
        ServerSink::new(sink, Json::default()),
    )
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct Point {
    x: f64,
    y: f64,
    z: f64,
}
// we could store this centrally in the server and use channels to push the current state into it.
// for example we could hold the current position and update it when success messages are passed
pub struct ClientState {
    last_message: String,
    current_position: Point,
}

impl From<ClientOnConnect> for ClientState {
    fn from(message: ClientOnConnect) -> Self {
        Self {
            last_message: message.message,
            current_position: message.current_position,
        }
    }
}

impl From<ClientCommandResponse> for ClientState {
    fn from(message: ClientCommandResponse) -> Self {
        Self {
            last_message: message.message,
            current_position: message.current_position,
        }
    }
}

impl From<ClientFailed> for ClientState {
    fn from(message: ClientFailed) -> Self {
        Self {
            last_message: message.server_command,
            current_position: message.current_position,
        }
    }
}
