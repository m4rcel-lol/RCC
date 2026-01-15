#[tokio::test]
async fn test_multi_player_sync() {
    // 1. Connect Client A
    let (mut ws_a, _) = connect_async("ws://localhost:8080").await.unwrap();
    // 2. Connect Client B
    let (mut ws_b, _) = connect_async("ws://localhost:8080").await.unwrap();

    // 3. Send Move from A
    let move_msg = r#"{"id":"A", "dx": 5, "dz": 0}"#;
    ws_a.send(Message::Text(move_msg.into())).await.unwrap();

    // 4. Verify B receives A's position update
    if let Some(Ok(Message::Text(resp))) = ws_b.next().await {
        assert!(resp.contains("\"x\":5"));
        println!("✅ Integration Test Passed: Movement Synced");
    }
}
