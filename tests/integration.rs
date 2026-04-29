use bbstore::{BBStoreConfig, Client};
use tokio::net::TcpListener;

fn config() -> BBStoreConfig {
    BBStoreConfig {
        num_shards: 2,
        address: "127.0.0.1".into(),
        buffer_size: 10,
    }
}

async fn start_server() -> std::net::SocketAddr {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(bbstore::run(listener, config()));
    addr
}

#[tokio::test]
async fn set_and_get() {
    let addr = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();

    client.set("mykey", "myvalue").await.unwrap();
    assert_eq!(client.get("mykey").await.unwrap(), Some("myvalue".to_string()));
}

#[tokio::test]
async fn get_missing_key() {
    let addr = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();

    assert_eq!(client.get("missing").await.unwrap(), None);
}

#[tokio::test]
async fn set_overwrites_existing_key() {
    let addr = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();

    client.set("k", "v1").await.unwrap();
    client.set("k", "v2").await.unwrap();
    assert_eq!(client.get("k").await.unwrap(), Some("v2".to_string()));
}

#[tokio::test]
async fn set_value_with_spaces() {
    let addr = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();

    client.set("k", "hello world").await.unwrap();
    assert_eq!(client.get("k").await.unwrap(), Some("hello world".to_string()));
}

#[tokio::test]
async fn multiple_commands_on_same_connection() {
    let addr = start_server().await;
    let mut client = Client::connect(addr).await.unwrap();

    for i in 0..10 {
        client.set(&format!("key-{}", i), &format!("val-{}", i)).await.unwrap();
    }
    for i in 0..10 {
        assert_eq!(
            client.get(&format!("key-{}", i)).await.unwrap(),
            Some(format!("val-{}", i))
        );
    }
}

#[tokio::test]
async fn multiple_clients_independent() {
    let addr = start_server().await;
    let mut c1 = Client::connect(addr).await.unwrap();
    let mut c2 = Client::connect(addr).await.unwrap();

    c1.set("shared", "v1").await.unwrap();
    assert_eq!(c2.get("shared").await.unwrap(), Some("v1".to_string()));
}
