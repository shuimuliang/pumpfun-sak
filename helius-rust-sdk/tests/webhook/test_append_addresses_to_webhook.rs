use helius::config::Config;
use helius::error::Result;
use helius::rpc_client::RpcClient;
use helius::types::{Cluster, HeliusEndpoints, TransactionType, Webhook, WebhookType};
use helius::Helius;
use mockito::Server;
use reqwest::Client;
use std::sync::Arc;

#[tokio::test]
async fn test_append_addresses_to_webhook_success() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());

    let mock_get_webhook_by_id_response: Webhook = Webhook {
        webhook_url: "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec![],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        wallet: "9Jt8mC9HXvh2g5s3PbTsNU71RS9MXUbhEMEmLTixYirb".to_string(),
        ..Default::default()
    };

    server
        .mock("GET", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_get_webhook_by_id_response).unwrap())
        .create();

    let mock_response: Webhook = Webhook {
        webhook_url: "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec!["71WDyyCsZwyEYDV91Qrb212rdg6woCHYQhFnmZUBxiJ6".to_string()],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        wallet: "9Jt8mC9HXvh2g5s3PbTsNU71RS9MXUbhEMEmLTixYirb".to_string(),
        ..Default::default()
    };

    server
        .mock("PUT", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
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

    let response: Result<Webhook> = helius
        .append_addresses_to_webhook(
            "0e8250a1-ceec-4757-ad69",
            &["71WDyyCsZwyEYDV91Qrb212rdg6woCHYQhFnmZUBxiJ6".to_string()],
        )
        .await;

    assert!(response.is_ok(), "The API call failed: {:?}", response.err());
    let webhook_response: Webhook = response.unwrap();

    assert_eq!(webhook_response.webhook_id, "0e8250a1-ceec-4757-ad69");
    assert_eq!(
        webhook_response.webhook_url,
        "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc"
    );
    assert_eq!(
        webhook_response.account_addresses,
        ["71WDyyCsZwyEYDV91Qrb212rdg6woCHYQhFnmZUBxiJ6".to_string()]
    )
}

#[tokio::test]
async fn test_append_addresses_to_webhook_failure() {
    let mut server: Server = Server::new_with_opts_async(mockito::ServerOpts::default()).await;
    let url: String = format!("{}/", server.url());
    let mock_get_webhook_by_id_response: Webhook = Webhook {
        webhook_url: "https://webhook.site/0e8250a1-ceec-4757-ad69-cc6473085bfc".to_string(),
        transaction_types: vec![TransactionType::Any],
        account_addresses: vec![],
        webhook_type: WebhookType::Enhanced,
        auth_header: None,
        webhook_id: "0e8250a1-ceec-4757-ad69".to_string(),
        wallet: "9Jt8mC9HXvh2g5s3PbTsNU71RS9MXUbhEMEmLTixYirb".to_string(),
        ..Default::default()
    };

    server
        .mock("GET", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
        .with_status(200)
        .with_header("Content-Type", "application/json")
        .with_body(serde_json::to_string(&mock_get_webhook_by_id_response).unwrap())
        .create();

    server
        .mock("PUT", "/v0/webhooks/0e8250a1-ceec-4757-ad69?api-key=fake_api_key")
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
    let response: Result<Webhook> = helius
        .append_addresses_to_webhook(
            "0e8250a1-ceec-4757-ad69",
            &["71WDyyCsZwyEYDV91Qrb212rdg6woCHYQhFnmZUBxiJ6".to_string()],
        )
        .await;
    assert!(response.is_err(), "Expected an error due to server failure");
}
