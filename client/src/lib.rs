use crate::agent::Agent;
use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use server::{ClientCommandResponse, ClientMessage, ClientOnConnect, ServerMessage};
use std::net::SocketAddr;
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::TcpStream;
use tokio_serde::{formats::Json, Framed};
use tokio_util::codec::{FramedRead, FramedWrite, LengthDelimitedCodec};

mod agent;
pub struct ClientBuilder {}

impl ClientBuilder {
    pub fn from_env() -> Result<Client> {
        let client_name =
            std::env::var("CLIENT_NAME").unwrap_or_else(|_| "test_client".to_string());

        let ip_address =
            std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:17653".to_string());
        let ip_address = ip_address.parse::<SocketAddr>()?;
        Ok(Client {
            client_name,
            ip_address,
            agent: Agent {
                current_position: Default::default(),
            },
        })
    }
}

type WrappedStream = FramedRead<OwnedReadHalf, LengthDelimitedCodec>;
type WrappedSink = FramedWrite<OwnedWriteHalf, LengthDelimitedCodec>;
type SerStream = Framed<WrappedStream, ServerMessage, (), Json<ServerMessage, ()>>;
type DeSink = Framed<WrappedSink, (), ClientMessage, Json<(), ClientMessage>>;

// This is provides some reasonable ergonomics around working with tcp.
// https://github.com/carllerche/tokio-serde/blob/master/examples/server.rs
fn wrap_stream(stream: TcpStream) -> (SerStream, DeSink) {
    // here we first split the stream into read and write this will allow us to work with them each seperately
    let (read, write) = stream.into_split();

    // here were wrapping them is a framed and length delimited codec.
    // this let's us not have to worry about buffering and provides deserialization using serde
    let stream = WrappedStream::new(read, LengthDelimitedCodec::new());
    let sink = WrappedSink::new(write, LengthDelimitedCodec::new());
    (
        SerStream::new(stream, Json::default()),
        DeSink::new(sink, Json::default()),
    )
}

pub struct Client {
    pub agent: Agent,
    pub client_name: String,
    pub ip_address: SocketAddr,
}

impl Client {
    pub async fn run(&mut self) -> Result<()> {
        // Bind a server socket
        let socket = TcpStream::connect(&self.ip_address).await
            .map_err(|_err| anyhow!("failed to connect"))?;


        // this is the same idea as the server. We're going t
        let (mut stream, mut sink) = wrap_stream(socket);

        // here we're going to use our sink to send the on connect message
        sink.send(ClientMessage::ClientOnConnect(ClientOnConnect {
            client_name: self.client_name.clone(),
            message: "hello".to_string(),
            current_position: self.agent.current_position.clone(),
        }))
        .await?;

        // now we can await an on connect message from the Server
        let message = stream
            .next()
            .await
            .ok_or_else(|| anyhow!("failed to retrieve next message"))?
            .map_err(|_err| anyhow!("failed to parse next message"))?;

        // lets read the on connect message from the server
        if let ServerMessage::OnConnect(message) = message {
            println!("client name: {}", message.client_name);
        } else {
            return Err(anyhow!("server sent wrong message"));
        };

        // the style of the server client is a response based.
        // here we will listen to commands, perform then and then respond
        loop {
            let message = stream
                .next()
                .await
                .ok_or_else(|| anyhow!("failed to retrieve next message"))?
                .map_err(|_err| anyhow!("failed to parse next message"))?;

            match message {
                ServerMessage::OnConnect(_) => {
                    return Err(anyhow!("server sent wrong message"));
                }
                ServerMessage::ServerMoveCommand(point) => {
                    println!("moving client to {:?}", point);
                    // update the state of the client
                    self.agent.update_position(point);
                    // once the above completes send the respone:
                    // here we're going to use our sink to send the on connect message
                    sink.send(ClientMessage::ClientCommandResponse(
                        ClientCommandResponse {
                            message: "success".to_string(),
                            current_position: self.agent.current_position.clone(),
                        },
                    ))
                    .await?;
                }
                ServerMessage::ServerFailed(_) => {
                    // our server has failed. we should probably start a reconnect loop
                    // for now we'll exit
                    return Err(anyhow!("server failed"));
                }
            }
        }
    }
}
