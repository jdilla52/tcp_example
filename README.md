# tcp server + client example 🚀
This was put together to demo setting up a tcp server in rust. At a high level his goal is to support a single server sending commands to clients and retrieving data from them. This is a simplified demo but should some intermediate functionality rust, tokio and serde provide for interacting with tcp.

The scenerio is as though the client is the remote control car and server is driving it around.

This was mostly taken from:
- [tokio](https://github.com/carllerche/tokio-serde/tree/master/examples)
- [jobq](https://github.com/cetra3/jobq)

Important libraries
- [serde](https://serde.rs/) an excellent parsing library
- [tokio](https://github.com/tokio-rs/tokio/tree/master) an async runtime which has man primitives for tcp
- [tokio-serde](https://github.com/carllerche/tokio-serde) provides some nice helpers for applying serde to the tokio stream
- [tokio-util](https://github.com/tokio-rs/tokio/tree/master/tokio-util) provides codecs for parsing the tokio stream

## Structure
This is a workspace with two crates:
### Server:
this holds the message types and the server implementation 
the types can be found in lib. They outline the communication from the server and client. These are used both in the server and the client. 

The server implementation is pretty straightforward. Its meant to show a couple of aspects of implementing a server like this:
- A store: the `TcpServer` holds a thread safe hashmap for storing client state, this is super common.
- `OnConnect` message: It's useful to have some sort of on connect message so metadata doesn't necessarily need to be sent in every single message.

At the moment several simple commands are hard coded into the server in `server.rs`. This would be easy to swap our or expose if desired.
```    
let movements = vec![
    Point {x: 0.0,y: 0.0,z: 0.0,},
    Point {x: 0.0,y: 0.0,z: 0.2,},
    Point {x: 0.0,y: 0.0,z: 0.4,},
    Point {x: 0.0,y: 0.0,z: 0.6,},
];
```

### Client:
the client is very simple. It's meant to show some form of local state which it will update based on the request the server. It first performs on connect and then responds to server commands.

## Up and running:
the easiest way to run the example:
first `cargo run --bin server`
second `cargo run --bin client`

in the client you should see it moving through the server commands:
```
client name: test_client
moving client to Point { x: 0.0, y: 0.0, z: 0.0 }
moving client to Point { x: 0.0, y: 0.0, z: 0.2 }
moving client to Point { x: 0.0, y: 0.0, z: 0.4 }
moving client to Point { x: 0.0, y: 0.0, z: 0.6 }
```

in the server you should see
```
Listening on: 127.0.0.1:17653
client connection dropped, 
position: Point { x: 0.0, y: 0.0, z: 0.6 }
last message: "success"
```

### Next Steps:
- implement the Stream trait to further improve ergonomics around retrieving information