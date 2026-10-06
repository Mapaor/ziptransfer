use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::UdpSocket;
use webrtc_turn::auth::LongTermAuthHandler;
use webrtc_turn::relay::relay_none::RelayAddressGeneratorNone;
use webrtc_turn::server::Server;
use webrtc_turn::server::config::{ConnConfig, ServerConfig};
use webrtc_util::vnet::net::Net;

pub async fn start_turn_server() {
    // 1. Create a UDP socket for the TURN server to listen on
    let conn = Arc::new(UdpSocket::bind("0.0.0.0:3478").await.unwrap());

    // 2. Set up long-term authentication
    // Note: In production, you might want to fetch these from your SQLite DB
    // or securely pass them via .env variables instead of hardcoding.
    let mut users = HashMap::new();
    let username = "ziptransfer_user";
    let password = "ziptransfer_password";
    let realm = "transfer.yourdomain.com";

    users.insert(
        username.to_owned(),
        webrtc_turn::auth::generate_auth_key(username, realm, password),
    );
    let auth_handler = Arc::new(LongTermAuthHandler::new(users));

    // 3. Configure the TURN Server
    let config = ServerConfig {
        conn_configs: vec![ConnConfig {
            conn,
            relay_addr_generator: Box::new(RelayAddressGeneratorNone {
                address: "127.0.0.1".to_owned(), // Your server's public IP goes here
                net: Arc::new(Net::new(None)),
            }),
        }],
        realm: realm.to_owned(),
        auth_handler: Arc::new(Box::new(auth_handler)), // Box the trait implementation
        channel_bind_timeout: std::time::Duration::from_secs(600),
    };

    // 4. Start the server
    tracing::info!("Starting Embedded TURN Server on 0.0.0.0:3478");
    let _server = Server::new(config).await.unwrap();

    // Keep the task alive
    tokio::signal::ctrl_c().await.unwrap();
}
