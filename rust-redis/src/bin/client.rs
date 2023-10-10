use bytes::Bytes;
use tokio::sync::oneshot;

#[derive(Debug)]
enum Command {
    Get { 
        key: String,
        resp: Responder<Option<Bytes>>, 
    },
    Set { 
        key: String, 
        val: Bytes,
        resp: Responder<()>,
    },
}

// manger can use this to send the result of a command back to the requester
type Responder<T> = oneshot::Sender<mini_redis::Result<T>>;

use mini_redis::client;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    // use clone to create a second handle to the transmitter
    let tx2 = tx.clone();

    let manager = tokio::spawn(async move {
        // Establish a connection to the server
        let mut client = client::connect("127.0.0.1:6379").await.unwrap();

        while let Some(cmd) = rx.recv().await {
            use Command::*;

            match cmd {
                Get { key, resp} => {
                    let res = client.get(&key).await;
                    let _ = resp.send(res);
                }
                Set { key, val, resp } => {
                    let res = client.set(&key, val).await;
                    let _ = resp.send(res);
                }
            }
        }
    });

    let t1 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Get {
            key: "foo".to_string(),
            resp: resp_tx,
        };

        // send the GET request
        tx.send(cmd).await.unwrap();

        // wait for the response
        let res = resp_rx.await;
        println!("GOT (Get) = {:?}", res);
    });

    let t2 = tokio::spawn(async move {
        let (resp_tx, resp_rx) = oneshot::channel();
        let cmd = Command::Set {
            key: "foo".to_string(),
            val: "bar".into(),
            resp: resp_tx,
        };

        // send the SET request
        tx2.send(cmd).await.unwrap();

        // wait for the response
        let res = resp_rx.await;
        println!("GOT (Set) = {:?}", res);
    });

    t1.await.unwrap();
    t2.await.unwrap();
    manager.await.unwrap();
}
