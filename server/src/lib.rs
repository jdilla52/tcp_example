use anyhow::Result;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use server::TcpServer;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

pub mod server;

#[derive(Deserialize, Debug)]
pub struct ClientFailed {
    server_command: String,
    current_position: Point,
}

#[derive(Deserialize, Debug)]
pub struct ClientOnConnect {
    client_name: String,
    message: String,
    current_position: Point,
}

#[derive(Deserialize, Debug)]
pub struct ClientCommandResponse {
    message: String,
    current_position: Point,
    // Here we could add an a set of error codes
}

// These are the messages the client will send to the server
#[derive(Deserialize, Debug)]
pub enum ClientMessage {
    ClientOnConnect(ClientOnConnect),
    ClientCommandResponse(ClientCommandResponse),
    Failed(ClientFailed),
}

#[derive(Serialize, Debug)]
pub struct ServerOnConnect {
    client_name: String,
    message: String,
}

#[derive(Serialize, Debug)]
pub struct ServerCommand {
    success_message: String,
    message_id: String,
}

// These are the messages the server will send to the client
#[derive(Serialize, Debug)]
pub enum ServerMessage {
    OnConnect(ServerOnConnect),
    ServerMoveCommand(Point),
    ServerFailed(String),
}

type WrappedStream = FramedRead<OwnedReadHalf, LengthDelimitedCodec>;
type WrappedSink = FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>;

// We use the unit type in place of the message types since we're
// only dealing with one half of the IO
type SerStream = Framed<WrappedStream, ClientMessage, (), Json<ClientMessage, ()>>;
type DeSink = Framed<WrappedSink, (), ServerMessage, Json<(), ServerMessage>>;

fn wrap_stream(stream: TcpStream) -> (SerStream, DeSink) {
    let (read, write) = stream.into_split();
    let stream = WrappedStream::new(read, LengthDelimitedCodec::new());
    let sink = WrappedSink::new(write, LengthDelimitedCodec::new());
    (
        SerStream::new(stream, Json::default()),
        DeSink::new(sink, Json::default()),
    )
}

#[derive(Deserialize, Serialize, Debug)]
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
