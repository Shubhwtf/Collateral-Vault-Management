//! Integration tests for the vault backend
//!
//! These tests require the backend server to be running on localhost:8080
//! Start it with `cargo run` before running tests

use reqwest;
use serde_json::json;
use std::time::Duration;

const BASE_URL: &str = "http://localhost:8080";

async fn check_server_available() -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();
    
    client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await
        .is_ok()
}

macro_rules! require_server {
    () => {
        if !check_server_available().await {
            eprintln!("\n⚠️  Backend server is not running on {}", BASE_URL);
            eprintln!("   Start the server with: cargo run");
            eprintln!("   Then run tests with: cargo test --test integration_test\n");
            return;
        }
    };
}

#[tokio::test]
async fn test_health_check() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(body["status"], "healthy");
    assert!(body.get("timestamp").is_some());
}

#[tokio::test]
async fn test_public_config() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/config/public", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("program_id").is_some());
    assert!(body.get("usdt_mint").is_some());
    assert!(body.get("solana_rpc_url").is_some());
}

#[tokio::test]
#[ignore]
async fn test_build_initialize_transaction() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/vault/initialize", BASE_URL))
        .json(&json!({
            "user_pubkey": "11111111111111111111111111111111"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success() || response.status().is_client_error());
    
    if response.status().is_success() {
        let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
        assert!(body.get("transaction_base64").is_some());
        assert!(body.get("recent_blockhash").is_some());
        assert!(body.get("fee_payer").is_some());
    }
}

#[tokio::test]
#[ignore]
async fn test_build_deposit_transaction() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/vault/deposit", BASE_URL))
        .json(&json!({
            "user_pubkey": "11111111111111111111111111111111",
            "amount": 1000000
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
#[ignore]
async fn test_build_withdraw_transaction() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/vault/withdraw", BASE_URL))
        .json(&json!({
            "user_pubkey": "11111111111111111111111111111111",
            "amount": 500000
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success() || response.status().is_client_error());
}

#[tokio::test]
#[ignore]
async fn test_get_balance_not_found() {
    require_server!();
    
    let client = reqwest::Client::new();
    let fake_pubkey = "11111111111111111111111111111111";
    
    let response = client
        .get(&format!("{}/vault/balance/{}", BASE_URL, fake_pubkey))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 404);
}

#[tokio::test]
#[ignore]
async fn test_get_transactions() {
    require_server!();
    
    let client = reqwest::Client::new();
    let fake_pubkey = "11111111111111111111111111111111";
    
    let response = client
        .get(&format!("{}/vault/transactions/{}", BASE_URL, fake_pubkey))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_success() || response.status() == 404);
}

#[tokio::test]
async fn test_get_tvl() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/vault/tvl", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("total_value_locked").is_some());
}

#[tokio::test]
async fn test_analytics_overview() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/analytics/overview", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("total_value_locked").is_some());
    assert!(body.get("total_users").is_some());
    assert!(body.get("active_vaults").is_some());
}

#[tokio::test]
async fn test_analytics_distribution() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/analytics/distribution", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.is_array());
}

#[tokio::test]
async fn test_analytics_utilization() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/analytics/utilization", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.get("total_collateral").is_some());
    assert!(body.get("utilization_rate").is_some());
}

#[tokio::test]
async fn test_analytics_tvl_chart() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .get(&format!("{}/analytics/chart/tvl?days=7", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.expect("Failed to parse JSON");
    assert!(body.is_array());
}

#[tokio::test]
async fn test_invalid_pubkey_format() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/vault/initialize", BASE_URL))
        .json(&json!({
            "user_pubkey": "invalid_pubkey"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_zero_amount_deposit() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/vault/deposit", BASE_URL))
        .json(&json!({
            "user_pubkey": "11111111111111111111111111111111",
            "amount": 0
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_client_error());
}

#[tokio::test]
async fn test_missing_required_fields() {
    require_server!();
    
    let client = reqwest::Client::new();
    
    let response = client
        .post(&format!("{}/vault/deposit", BASE_URL))
        .json(&json!({
            "user_pubkey": "11111111111111111111111111111111"
        }))
        .send()
        .await
        .expect("Failed to send request");
    
    assert!(response.status().is_client_error());
}

// ignored by default because it hammers the server
// run with: cargo test test_concurrent_requests -- --ignored
#[tokio::test]
#[ignore]
async fn test_concurrent_requests() {
    require_server!();
    
    let client = reqwest::Client::new();
    let mut handles = vec![];
    
    for _ in 0..10 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            client
                .get(&format!("{}/health", BASE_URL))
                .send()
                .await
                .expect("Failed to send request")
        });
        handles.push(handle);
    }
    
    for handle in handles {
        let response = handle.await.expect("Task panicked");
        assert_eq!(response.status(), 200);
    }
}

#[tokio::test]
async fn test_response_time() {
    require_server!();
    
    let client = reqwest::Client::new();
    let start = std::time::Instant::now();
    
    let _response = client
        .get(&format!("{}/health", BASE_URL))
        .send()
        .await
        .expect("Failed to send request");
    
    let duration = start.elapsed();
    
    // health check should be fast - if it's not, something's wrong
    assert!(duration.as_millis() < 100, "Response time too slow: {:?}", duration);
}

