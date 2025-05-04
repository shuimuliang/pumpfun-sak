use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{Cluster, HeliusEndpoints, TransactionType, Webhook, WebhookType};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_get_all_webhooks_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_response: Vec<Webhook> = vec![Webhook {
        webhook_url: "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec![],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        ..Default::default()
    }];

    server
        .mock("GET", "/v0/webhooks?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_response).unwrap())
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    let response: Result<Vec<Webhook>> = helius.get_all_webhooks().await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    let webhook_response: Vec<Webhook> = response.unwrap();

    assert_eq!(webhook_response.len(), 1);
    assert_eq!(webhook_response[0].webhook_id, "0e8250a1-ceec-4757-ad69");
    assert_eq!(
        webhook_response[0].webhook_url,
        "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc"
    )
}

#[tokio::test]
async fn test_get_all_webhooks_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    server
        .mock("GET", "/v0/webhooks?api-key=fake_api_key")
        .with_status(500)
        .with_header("Content-Type", "application/json")
        .with_body(r#"{"error":"Internal Server Error"}"#)
        .create();

    let config: Arc<Config> = Arc::new(Config {
        api_key: "fake_api_key".to_string(),
        cluster: Cluster::Devnet,
        endpoints: HeliusEndpoints {
            api: url.to_string(),
            rpc: url.to_string(),
        },
    });

    let client: Client = Client::new();
    let rpc_client: Arc<RpcClient> = Arc::new(RpcClient::new(Arc::new(client.clone()), Arc::clone(&config)).unwrap());
    let helius: Helius = Helius {
        config,
        client,
        rpc_client,
        async_rpc_client: None,
        ws_client: None,
    };

    let response: Result<Vec<Webhook>> = helius.get_all_webhooks().await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
