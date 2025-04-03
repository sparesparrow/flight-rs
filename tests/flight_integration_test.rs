use anyhow::{Context, Result};
use flight_sim::{ClientInput, GameState};
use futures::{SinkExt, StreamExt};
use insta::assert_yaml_snapshot;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio::time::{timeout, Duration};
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use url::Url;

// Helper to connect to the WebSocket server
async fn connect_client(
    addr: SocketAddr,
) -> Result<(WebSocketStream<MaybeTlsStream<TcpStream>>, String)> {
    let url_str = format!("ws://{}/ws", addr);
    let url = Url::parse(&url_str)?;

    let (ws_stream, _response) = connect_async(url)
        .await
        .context("Failed to connect to WebSocket")?;

    // Wait for the welcome message and extract client ID
    let welcome_msg = timeout(Duration::from_secs(2), ws_stream.next())
        .await?? // Timeout error, then WebSocket error
        .context("Did not receive welcome message")?;

    let client_id = match welcome_msg {
        Message::Text(text) if text.starts_with("Welcome!") => text
            .split(":")
            .nth(1)
            .map(|s| s.trim().to_string())
            .context("Could not parse client ID from welcome message")?,
        _ => anyhow::bail!("Received unexpected message instead of welcome"),
    };

    Ok((ws_stream, client_id))
}

// Helper to send input
async fn send_input(
    ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>,
    input: ClientInput,
) -> Result<()> {
    let msg = serde_json::to_string(&input)?;
    ws.send(Message::Text(msg))
        .await
        .context("Failed to send input")
}

// Helper to receive and parse game state
async fn receive_state(ws: &mut WebSocketStream<MaybeTlsStream<TcpStream>>) -> Result<GameState> {
    let msg = timeout(Duration::from_secs(1), ws.next())
        .await?? // Timeout error, then WebSocket error
        .context("Did not receive game state message")?;

    match msg {
        Message::Text(text) => serde_json::from_str(&text).context("Failed to parse game state"),
        _ => anyhow::bail!("Received unexpected message type instead of game state"),
    }
}

// Integration test
#[tokio::test]
async fn test_basic_flight_maneuvers() -> Result<()> {
    // Start the server in a background task
    // Use a different port for testing to avoid conflicts
    let addr: SocketAddr = "127.0.0.1:8082".parse()?;
    tokio::spawn(flight_sim::run_server(addr));
    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(500)).await;

    // Connect a client
    let (mut ws, _client_id) = connect_client(addr).await?;

    // --- Test Scenario: Takeoff and Climb ---
    // Receive initial state
    let _initial_state = receive_state(&mut ws).await?;

    // Apply full throttle
    send_input(
        &mut ws,
        ClientInput {
            throttle_up: true,
            ..Default::default()
        },
    )
    .await?;
    // Wait for physics to update (a few frames)
    tokio::time::sleep(Duration::from_millis(100)).await;
    let _state_after_throttle = receive_state(&mut ws).await?;

    // Apply pitch up
    send_input(
        &mut ws,
        ClientInput {
            throttle_up: true,
            pitch_up: true,
            ..Default::default()
        },
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let state_after_climb = receive_state(&mut ws).await?;

    // --- Test Scenario: Level Flight ---
    // Reduce pitch input
    send_input(
        &mut ws,
        ClientInput {
            throttle_up: true,
            ..Default::default()
        },
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let state_after_level = receive_state(&mut ws).await?;

    // --- Test Scenario: Descent ---
    // Reduce throttle and pitch down
    send_input(
        &mut ws,
        ClientInput {
            throttle_down: true,
            pitch_down: true,
            ..Default::default()
        },
    )
    .await?;
    tokio::time::sleep(Duration::from_millis(200)).await;
    let state_after_descent = receive_state(&mut ws).await?;

    // --- Snapshot Testing ---
    // Use insta to capture the state at various points
    // Snapshots are stored in tests/snapshots/
    assert_yaml_snapshot!("takeoff_climb", state_after_climb);
    assert_yaml_snapshot!("level_flight", state_after_level);
    assert_yaml_snapshot!("descent", state_after_descent);

    // Close the connection gracefully
    ws.close(None).await?;

    Ok(())
}
