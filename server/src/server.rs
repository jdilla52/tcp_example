use crate::{ClientMessage, ClientState, Point, ServerMessage, ServerOnConnect};

use anyhow::{anyhow, Result};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

pub struct TcpBuilder {}

impl TcpBuilder {
    pub fn from_env() -> Result<TcpServer> {
        let ip_address =
            std::env::var("SERVER_ADDRESS").unwrap_or_else(|_| "127.0.0.1:17653".to_string());
        let ip_address = ip_address.parse::<SocketAddr>()?;
        Ok(TcpServer {
            ip_address,
            clients: Arc::new(Mutex::new(Default::default())),
        })
    }
}

type ClientStore = Arc<Mutex<HashMap<String, ClientState>>>;
pub struct TcpServer {
    ip_address: SocketAddr,
    // concurrent data structures in rust are excellent
    // here we can hold centrally the state of all of our clients
    clients: ClientStore,
}

impl TcpServer {
    pub async fn run(&self) -> anyhow::Result<()> {
        // Next up we create a TCP listener which will listen for incoming
        // connections. This TCP listener is bound to the address we determined
        let listener = TcpListener::bind(&self.ip_address).await?;
        println!("Listening on: {}", self.ip_address);

        // This is the connection loop where whenever a new client connects we move it
        // to a separate task so we dont block and protect the server from any failures.

        loop {
            // Here we're copying a reference to our client state.
            let client = self.clients.clone();
            // Asynchronously wait for an inbound socket.
            let (socket, _) = listener.accept().await?;
            // Delimit frames using a length header
            tokio::spawn(async move {
                match Self::handle_client(socket, client).await {
                    Ok(_) => {}
                    Err(err) => {
                        println!("error: {:?}", err)
                    }
                };
            });
        }
    }

    ///
    /// This method handles a client connection. We can think about the entry point for interacting with clients
    /// We can expand the types of messages and connections we might handle
    /// We can use serde to fully type and parse messages
    /// Here we can also imply state-fullness in the server by storing the current state of clients
    ///
    async fn handle_client(socket: TcpStream, clients_state: ClientStore) -> Result<()> {
        let (mut stream, mut sink) = crate::wrap_stream(socket);

        // this first example is about how to deal with an on connect message and storing some data into our global store.
        let mut current_client = String::new();
        // here we can register our new client storing it's state locally in this thread and globally in the store
        // this is where we would build the datastructure in the server to hold client side state
        // ie client a. moved to x.
        let message = stream
            .next()
            .await
            .expect("The connection message wasn't set")
            .expect("Failed to parse pong");

        // as the stream is monolithic these all need to be handled originating in one place:
        // Here we can parse the various types of messages the client might send us and respond.
        if let ClientMessage::ClientOnConnect(client) = message {
            // update the local state so the client no longer needs to send an id:
            current_client = client.client_name.clone();
            // here we're updating the server state with the message from the client
            let mut lock = clients_state.lock().await;
            lock.insert(current_client.clone(), client.into());
            // may as well explictly drop here. This allows other threads to interact with the store
            drop(lock);

            sink.send(ServerMessage::OnConnect(ServerOnConnect {
                client_name: current_client.clone(),
                message: "hello".to_string(),
            }))
            .await
            .expect("Failed to send ping to server");
        };

        // This example is about sending a set of commands and dealing with responses:

        // After a successfull connection let's now try to send some commands to the client
        // let's say we want to move all our clients to a couple of positions
        let movements = vec![
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.0,
            },
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.2,
            },
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.4,
            },
            Point {
                x: 0.0,
                y: 0.0,
                z: 0.6,
            },
        ];

        for point in movements {
            // here we're going to use our sink to send commands
            sink.send(ServerMessage::ServerMoveCommand(point))
                .await
                .expect("Failed to send ping to server");

            let message: ClientMessage = stream
                .next()
                .await
                .ok_or_else(|| anyhow!("failed to retrieve next message"))?
                .map_err(|_err| anyhow!("failed to parse next message"))?;

            match message {
                ClientMessage::ClientOnConnect(_) => {
                    // This should be an error
                    return Err(anyhow!("server sent wrong message"));
                }
                ClientMessage::ClientCommandResponse(client) => {
                    // update our local state
                    let mut lock = clients_state.lock().await;
                    lock.insert(current_client.clone(), client.into());
                    drop(lock);
                }
                ClientMessage::Failed(client) => {
                    // update our local state
                    let mut lock = clients_state.lock().await;
                    lock.insert(current_client.clone(), client.into());
                    drop(lock);

                    // here we could add retry logic when a client fails
                    return Err(anyhow!("client failed"));
                }
            }
        }
        let lock = clients_state.lock().await;
        let curr_client = lock
            .get(current_client.as_str())
            .ok_or_else(|| anyhow!("no client present in store"))?;
        println!(
            "client connection dropped, \n position: {:?}\n last message\n{:?}",
            curr_client.current_position, curr_client.last_message
        );
        Ok(())
    }
}
