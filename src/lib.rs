use futures::{SinkExt, StreamExt};
use log::{info, warn};
use nalgebra::Vector3;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::sync::mpsc;
use tokio_tungstenite::tungstenite::Message as TungsteniteMessage;
use uuid::Uuid;
use warp::{
    ws::{WebSocket, Ws},
    Filter,
};

// Import RPG structs FIRST to avoid naming conflicts during definition
pub mod rpg_structs;
pub use rpg_structs::*;

// Import physics code (might be repurposed for map navigation later)
// pub mod physics; // Assuming physics is defined elsewhere if needed, or remove if unused.
// use physics::Aircraft; // Remove if Aircraft physics are fully replaced

// Constants
const FRAME_TIME: f32 = 1.0 / 30.0; // RPG loop can be slower, 30 FPS equivalent tick rate

// --- Original Flight Sim Structs (Renamed) ---
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub struct FlightSimInput {
    // Renamed from ClientInput
    pub pitch_up: bool,
    pub pitch_down: bool,
    pub throttle_up: bool,
    pub throttle_down: bool,
}

// This might become redundant or merged into the RPG GameState
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FlightSimState {
    // Renamed from GameState
    pub aircraft: Vec<FlightSimAircraftState>,
}

// This might become redundant or represented differently
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FlightSimAircraftState {
    // Renamed from AircraftState
    pub id: String, // Keep using String for ID here for now
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub theta: f32,
    pub throttle_level: f32,
}
// --- End Renamed Structs ---

// --- RPG Shared State Types ---
// Map Player UUID to their Character state
pub type CharacterMap = Arc<Mutex<HashMap<Uuid, Character>>>;
// Map Player UUID to their WebSocket sender channel
pub type Clients = Arc<Mutex<HashMap<Uuid, mpsc::UnboundedSender<TungsteniteMessage>>>>;
// Shared overall game state (including world state)
pub type SharedGameState = Arc<Mutex<GameState>>; // Using the RPG GameState

// Helper functions to inject shared state into route handlers
fn with_clients(
    clients: Clients,
) -> impl Filter<Extract = (Clients,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || clients.clone())
}

// Inject CharacterMap instead of AircraftMap
fn with_characters(
    characters: CharacterMap,
) -> impl Filter<Extract = (CharacterMap,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || characters.clone())
}

// Inject SharedGameState
fn with_game_state(
    game_state: SharedGameState,
) -> impl Filter<Extract = (SharedGameState,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || game_state.clone())
}

// Handle new WebSocket connections
async fn handle_connection(
    ws: WebSocket,
    clients: Clients,
    // characters: CharacterMap, // Characters are now part of SharedGameState
    game_state: SharedGameState,
) {
    let player_id = Uuid::new_v4(); // Use Uuid directly
    info!("New connection attempt: {}", player_id);

    let (mut ws_sender, mut ws_receiver) = ws.split();
    let (client_sender, mut client_receiver) = mpsc::unbounded_channel::<TungsteniteMessage>();

    // Add client sender to the map immediately
    clients.lock().unwrap().insert(player_id, client_sender);

    // Send initial Welcome message - Client needs to send CharacterCreation request
    // We no longer create a default character/aircraft here.
    let initial_state = game_state.lock().unwrap().clone(); // Clone the current state
    let welcome_msg = ServerMessage::Welcome {
        player_id,
        initial_game_state: initial_state,
    };

    if let Ok(serialized_welcome) = serde_json::to_string(&welcome_msg) {
        if ws_sender
            .send(warp::ws::Message::text(serialized_welcome))
            .await
            .is_err()
        {
            warn!(
                "Failed to send welcome message to potential client {}",
                player_id
            );
            clients.lock().unwrap().remove(&player_id); // Clean up sender if welcome fails
            return;
        }
        info!("Sent Welcome to potential client {}", player_id);
    } else {
        warn!(
            "Failed to serialize welcome message for potential client {}",
            player_id
        );
        clients.lock().unwrap().remove(&player_id); // Clean up
        return;
    }

    // Task to forward messages from game loop/server logic to this client's WebSocket
    let forward_player_id = player_id; // Clone for the task
    let forward_clients = clients.clone();
    // let forward_characters = characters.clone(); // Pass characters map if needed later
    let forward_game_state = game_state.clone();
    tokio::spawn(async move {
        while let Some(message_to_send) = client_receiver.recv().await {
            // message_to_send is TungsteniteMessage
            let warp_message = match message_to_send {
                TungsteniteMessage::Text(t) => warp::ws::Message::text(t),
                TungsteniteMessage::Binary(b) => warp::ws::Message::binary(b),
                TungsteniteMessage::Ping(p) => warp::ws::Message::ping(p),
                TungsteniteMessage::Pong(p) => warp::ws::Message::pong(p),
                TungsteniteMessage::Close(_) => warp::ws::Message::close(),
                TungsteniteMessage::Frame(_) => continue, // Skip raw frames
            };

            if ws_sender.send(warp_message).await.is_err() {
                warn!(
                    "Failed to send message to client {}, disconnecting task",
                    forward_player_id
                );
                // Trigger disconnect logic from here if send fails
                handle_disconnect(forward_player_id, &forward_clients, &forward_game_state);
                break;
            }
        }
        // If the loop ends (e.g., channel closed), ensure disconnect logic is called
        warn!("Forwarding task for {} ending.", forward_player_id);
        // Maybe call handle_disconnect here too? Depends if recv() returning None means disconnect
    });

    // Process incoming messages from this client
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(message) => {
                // Incoming message is warp::ws::Message
                if message.is_text() {
                    let msg_str = message.to_str().unwrap_or_default();
                    match serde_json::from_str::<ClientMessage>(msg_str) {
                        Ok(client_msg) => {
                            // Handle the deserialized ClientMessage
                            // Acquire lock ONCE per message if possible
                            let mut state_guard = game_state.lock().unwrap();
                            handle_client_message(
                                player_id,
                                client_msg,
                                &mut state_guard,
                                &clients,
                            );
                        }
                        Err(e) => {
                            // Log unrecognized text messages that aren't valid ClientMessage JSON
                            warn!("Failed to deserialize text message from client {}: {}. Content: '{}'", player_id, e, msg_str);
                            // Optionally send an error back to the client
                            let error_msg =
                                ServerMessage::Error(format!("Invalid message format: {}", e));
                            if let Ok(json_err) = serde_json::to_string(&error_msg) {
                                if let Some(sender) = clients.lock().unwrap().get(&player_id) {
                                    let _ = sender.send(TungsteniteMessage::Text(json_err));
                                }
                            }
                        }
                    }
                } else if message.is_binary() {
                    warn!(
                        "Received unexpected binary message from client {}",
                        player_id
                    );
                    // Handle binary data if needed, otherwise ignore or error
                } else if message.is_ping() {
                    // Warp handles pongs automatically, but you can log if needed
                    info!("Received ping from client {}", player_id);
                } else if message.is_pong() {
                    info!("Received pong from client {}", player_id);
                    // Handle pong if you are manually sending pings
                } else if message.is_close() {
                    info!("Received close frame from client {}", player_id);
                    break; // Exit loop on close message
                }
            }
            Err(e) => {
                warn!("WebSocket error for client {}: {}", player_id, e);
                break;
            }
        }
    }

    // Client disconnected (loop exited)
    handle_disconnect(player_id, &clients, &game_state);
}

// Function to process validated client messages
fn handle_client_message(
    player_id: Uuid,
    msg: ClientMessage,
    game_state: &mut GameState, // Mutably borrow the GameState
    clients: &Clients,          // To broadcast updates
) {
    info!("Received message from {}: {:?}", player_id, msg);

    match msg {
        ClientMessage::RequestCharacterCreation { name, occupation } => {
            if !game_state.players.contains_key(&player_id) {
                let mut new_char = Character::new(player_id, name.clone(), occupation.clone());

                // Adjust stats based on occupation (example)
                match occupation.as_str() {
                    "Records Department Worker" => {
                        new_char.loyalty = new_char.loyalty.saturating_sub(5);
                        new_char.thoughtcrime = new_char.thoughtcrime.saturating_add(10);
                    }
                    "Junior Spy Instructor" => {
                        new_char.loyalty = new_char.loyalty.saturating_add(15);
                        new_char.suspicion = new_char.suspicion.saturating_sub(10);
                    }
                    "Fiction Department Writer" => {
                        new_char.thoughtcrime = new_char.thoughtcrime.saturating_add(15);
                    }
                    _ => { // Default or unknown occupation
                    }
                }

                info!(
                    "Creating character '{}' ({}) for player {}",
                    name, occupation, player_id
                );
                let char_clone = new_char.clone(); // Clone for broadcast message
                game_state.players.insert(player_id, new_char);

                // Notify all OTHER clients that a new player joined
                let join_msg = ServerMessage::PlayerJoined {
                    player_id,
                    character: char_clone,
                };
                broadcast_message(&clients, Some(&player_id), &join_msg); // Send to everyone except the new player

                // Send the full updated state back to the new player (confirmation)
                let update_msg = ServerMessage::GameStateUpdate(game_state.clone());
                send_message_to_client(&clients, player_id, &update_msg);
            } else {
                warn!(
                    "Player {} tried to create character but already exists.",
                    player_id
                );
                let error_msg = ServerMessage::Error("Character already created.".to_string());
                send_message_to_client(&clients, player_id, &error_msg);
            }
        }
        ClientMessage::MoveRequest { target_location } => {
            if let Some(character) = game_state.players.get_mut(&player_id) {
                let current_location_name = &character.location;
                if let Some(current_loc_details) =
                    game_state.world_state.locations.get(current_location_name)
                {
                    if current_loc_details.connections.contains(&target_location) {
                        if game_state
                            .world_state
                            .locations
                            .contains_key(&target_location)
                        {
                            info!(
                                "Player {} moving from {} to {}",
                                player_id, character.location, target_location
                            );
                            character.location = target_location;
                            // TODO: Add travel risk check? Random events on move?
                            // Broadcast the change
                            broadcast_state_update(&clients, game_state);
                        } else {
                            warn!(
                                "Player {} tried to move to invalid location {}",
                                player_id, target_location
                            );
                            let error_msg = ServerMessage::Error(format!(
                                "Invalid move target: {}",
                                target_location
                            ));
                            send_message_to_client(&clients, player_id, &error_msg);
                        }
                    } else {
                        warn!(
                            "Player {} tried invalid move from {} to {}",
                            player_id, current_location_name, target_location
                        );
                        let error_msg = ServerMessage::Error(format!(
                            "Cannot move from {} to {}",
                            current_location_name, target_location
                        ));
                        send_message_to_client(&clients, player_id, &error_msg);
                    }
                } else {
                    warn!(
                        "Player {} is in an invalid current location: {}",
                        player_id, current_location_name
                    );
                    let error_msg = ServerMessage::Error(
                        "Internal server error: Current location invalid.".to_string(),
                    );
                    send_message_to_client(&clients, player_id, &error_msg);
                }
            } else {
                warn!("MoveRequest from unknown player {}", player_id);
                // Ignore or send error?
            }
        }
        ClientMessage::FlyInput {
            pitch,
            roll,
            yaw,
            throttle_change,
        } => {
            if let Some(character) = game_state.players.get_mut(&player_id) {
                info!(
                    "Received FlyInput from {}: P:{:.2} R:{:.2} Y:{:.2} T:{:.2}",
                    player_id, pitch, roll, yaw, throttle_change
                );

                // Update Throttle
                character.throttle =
                    (character.throttle + throttle_change * FRAME_TIME * 2.0).clamp(0.0, 1.0);

                // --- Basic Orientation Update ---
                let rotation_speed = 1.5 * FRAME_TIME; // Radians per second scaled by frame time
                let pitch_rad = pitch * rotation_speed;
                let roll_rad = roll * rotation_speed;
                let yaw_rad = yaw * rotation_speed;

                // Create quaternions for each axis rotation
                let pitch_axis = nalgebra::Unit::new_normalize(
                    character.orientation.transform_vector(&Vector3::x_axis()),
                );
                let roll_axis = nalgebra::Unit::new_normalize(
                    character.orientation.transform_vector(&Vector3::z_axis()),
                );
                let yaw_axis = nalgebra::Unit::new_normalize(
                    character.orientation.transform_vector(&Vector3::y_axis()),
                );

                let pitch_quat = nalgebra::UnitQuaternion::from_axis_angle(&pitch_axis, pitch_rad);
                let roll_quat = nalgebra::UnitQuaternion::from_axis_angle(&roll_axis, roll_rad);
                let yaw_quat = nalgebra::UnitQuaternion::from_axis_angle(&yaw_axis, yaw_rad);

                // Combine rotations
                character.orientation = yaw_quat * pitch_quat * roll_quat * character.orientation;
            } else {
                warn!("FlyInput from unknown player {}", player_id);
            }
        }
        ClientMessage::InteractRequest {
            npc_name,
            interaction_type,
        } => {
            info!(
                "Player {} trying to interact with {} (type {})",
                player_id, npc_name, interaction_type
            );
            // TODO: Implement NPC interaction logic based on rpg_structs.py
            // This will involve checking character location, NPC location, trust levels,
            // updating character stats, relationships, inventory, and sending NarrativeUpdates.
            let narrative = format!(
                "You interact with {}. (Interaction type {} - logic not implemented yet)",
                npc_name, interaction_type
            );
            let update_msg = ServerMessage::NarrativeUpdate(narrative);
            send_message_to_client(&clients, player_id, &update_msg);
            // Remember to broadcast state changes if interaction modifies public state
        }
        ClientMessage::JournalWriteRequest { entry } => {
            if let Some(character) = game_state.players.get_mut(&player_id) {
                info!("Player {} writing to journal.", player_id);
                character.journal_entries.push(entry);
                character.thoughtcrime = character.thoughtcrime.saturating_add(5); // Increase thoughtcrime
                                                                                   // TODO: Add risk of being caught?
                let narrative =
                    "You write in your secret journal. Your thoughtcrime increases.".to_string();
                let narrative_msg = ServerMessage::NarrativeUpdate(narrative);
                send_message_to_client(&clients, player_id, &narrative_msg);
                // Send updated stats privately
                broadcast_state_update(&clients, game_state); // Or send private update
            }
        }
        ClientMessage::SearchRequest => {
            info!("Player {} is searching.", player_id);
            // TODO: Implement search logic based on location, chance, stats
            let narrative =
                "You search the area, but find nothing of interest (logic not implemented yet)."
                    .to_string();
            let update_msg = ServerMessage::NarrativeUpdate(narrative);
            send_message_to_client(&clients, player_id, &update_msg);
        }
        ClientMessage::WorkRequest => {
            info!("Player {} is working.", player_id);
            // TODO: Implement work logic based on occupation, location
            let narrative =
                "You perform your duties for the Party (logic not implemented yet).".to_string();
            let update_msg = ServerMessage::NarrativeUpdate(narrative);
            send_message_to_client(&clients, player_id, &update_msg);
        }
        ClientMessage::RestRequest => {
            info!("Player {} rests.", player_id);
            // TODO: Implement rest logic (pass time, potential events)
            // For now, just advance the day if ALL players rest? Complex coordination needed.
            // Simplification: Maybe resting just recovers a bit of health?
            if let Some(character) = game_state.players.get_mut(&player_id) {
                character.health = character.health.saturating_add(5).min(100);
                let narrative = "You rest for a while, recovering slightly.".to_string();
                let narrative_msg = ServerMessage::NarrativeUpdate(narrative);
                send_message_to_client(&clients, player_id, &narrative_msg);
                broadcast_state_update(&clients, game_state); // Broadcast health change
            }
        }
    }
    // Note: Broadcasting the entire state on every action can be inefficient.
    // Consider sending targeted updates or deltas in a more complex implementation.
    // For now, broadcasting the whole state is simpler.
    // broadcast_state_update(&clients, game_state); // Moved inside handlers where state changes
}

// Helper to handle client disconnection logic
fn handle_disconnect(
    player_id: Uuid,
    clients: &Clients,
    game_state: &SharedGameState,
    // characters: &CharacterMap // Now part of game_state
) {
    info!("Client {} disconnected", player_id);
    clients.lock().unwrap().remove(&player_id);

    let mut state_guard = game_state.lock().unwrap();
    let removed_char = state_guard.players.remove(&player_id); // Remove player from game state

    if removed_char.is_some() {
        info!("Removed character data for player {}", player_id);
        // Notify remaining clients that the player left
        let leave_msg = ServerMessage::PlayerLeft { player_id };
        broadcast_message(&clients, Some(&player_id), &leave_msg); // Send to everyone else
    } else {
        info!(
            "Disconnect for player {} who hadn't created a character.",
            player_id
        );
    }
}

// Helper to send a ServerMessage to a specific client
fn send_message_to_client(clients: &Clients, player_id: Uuid, message: &ServerMessage) {
    if let Ok(serialized_msg) = serde_json::to_string(message) {
        let clients_map = clients.lock().unwrap();
        if let Some(sender) = clients_map.get(&player_id) {
            if sender
                .send(TungsteniteMessage::Text(serialized_msg))
                .is_err()
            {
                warn!(
                    "Failed to send message to client {} (already disconnected?)",
                    player_id
                );
                // Sender might be removed soon by disconnect handler
            }
        } else {
            warn!(
                "Attempted to send message to non-existent client {}",
                player_id
            );
        }
    } else {
        warn!("Failed to serialize message for client {}", player_id);
    }
}

// Helper: Broadcast Message to All Clients (Optionally Exclude One)
// Ensure the signature correctly uses Option<&Uuid>
fn broadcast_message(clients: &Clients, exclude_player_id: Option<&Uuid>, message: &ServerMessage) {
    match serde_json::to_string(message) {
        Ok(serialized_msg) => {
            let clients_map = clients.lock().unwrap();
            for (id, sender) in clients_map.iter() {
                // Send if not excluded
                if exclude_player_id.map_or(true, |exclude_id| id != exclude_id) {
                    if sender
                        .send(TungsteniteMessage::Text(serialized_msg.clone()))
                        .is_err()
                    {
                        warn!("Failed to broadcast to {} (already disconnected?)", id);
                        // Disconnect logic will handle cleanup.
                    }
                }
            }
        }
        Err(e) => {
            warn!("Failed to serialize broadcast message {:?}: {}", message, e);
        }
    }
}

// Helper to broadcast the entire game state
fn broadcast_state_update(clients: &Clients, game_state: &GameState) {
    let update_msg = ServerMessage::GameStateUpdate(game_state.clone());
    if let Ok(serialized_msg) = serde_json::to_string(&update_msg) {
        let clients_map = clients.lock().unwrap();
        for (id, sender) in clients_map.iter() {
            if sender
                .send(TungsteniteMessage::Text(serialized_msg.clone()))
                .is_err()
            {
                warn!(
                    "Failed to broadcast state update to client {} (already disconnected?)",
                    id
                );
            }
        }
    } else {
        warn!("Failed to serialize game state for broadcast update");
    }
}

// Main game loop - Now focused on RPG state updates, time progression, events
async fn game_loop(clients: Clients, game_state: SharedGameState) {
    let tick_duration = Duration::from_secs_f32(FRAME_TIME);
    info!(
        "Game loop started with tick rate: {} Hz ({:?})",
        1.0 / FRAME_TIME,
        tick_duration
    );

    loop {
        let loop_start_time = Instant::now();

        // --- Game Logic Tick ---
        let mut state_changed = false; // Track if state needs broadcasting
        {
            // Lock scope for game state modification
            let mut state_guard = game_state.lock().unwrap();

            // --- Time Progression ---
            // TODO: Implement day/date progression logic
            // state_guard.day += 1;
            // state_guard.world_state.current_date = calculate_new_date(state_guard.day);

            // --- Random World Events ---
            // TODO: Implement random events based on python code (e.g., ration changes, enemy changes, patrols)
            // Example:
            // if rand::thread_rng().gen_bool(0.01) { // 1% chance per tick
            //    state_guard.world_state.chocolate_ration = state_guard.world_state.chocolate_ration.saturating_sub(1);
            //    let narrative = ServerMessage::NarrativeUpdate("The chocolate ration has been reduced!".to_string());
            //    broadcast_message(&clients, &Uuid::nil(), &narrative); // Broadcast to all
            // }

            // --- NPC Movement/State Changes ---
            // TODO: Implement NPC logic (e.g., moving between locations)

            // --- Player Stat Decay/Changes ---
            // TODO: Implement passive changes (e.g., slight loyalty decrease over time?)

            // --- Check for Player End Conditions ---
            let mut players_to_remove = Vec::new();
            for (id, character) in state_guard.players.iter() {
                if character.health == 0 {
                    info!("Player {} ({}) has died.", id, character.name);
                    players_to_remove.push(*id);
                    let death_msg = ServerMessage::NarrativeUpdate(
                        "Your health reached zero. You succumb to the harsh realities of Oceania."
                            .to_string(),
                    );
                    send_message_to_client(&clients, *id, &death_msg);
                } else if character.suspicion >= 100 {
                    info!(
                        "Player {} ({}) has been arrested by the Thought Police!",
                        id, character.name
                    );
                    players_to_remove.push(*id);
                    let arrest_msg = ServerMessage::NarrativeUpdate("Your suspicion level reached its peak. You are arrested by the Thought Police and taken to the Ministry of Love. Your journey ends here.".to_string());
                    send_message_to_client(&clients, *id, &arrest_msg);
                }
            }

            // Remove players who met end conditions
            let mut _player_left_during_tick = false;
            for id_to_remove in players_to_remove {
                if state_guard.players.remove(&id_to_remove).is_some() {
                    let leave_msg = ServerMessage::PlayerLeft {
                        player_id: id_to_remove,
                    };
                    broadcast_message(&clients, Some(&id_to_remove), &leave_msg);
                    state_changed = true;
                    _player_left_during_tick = true;

                    if let Some(sender) = clients.lock().unwrap().get(&id_to_remove) {
                        let _ = sender.send(TungsteniteMessage::Close(None));
                        info!("Sent close message to removed player {}", id_to_remove);
                    }
                }
            }

            // --- 3D Physics Update ---
            let gravity = Vector3::new(0.0, -9.81, 0.0);
            let drag_coefficient = 0.5; // Simple linear drag

            for (_id, character) in state_guard.players.iter_mut() {
                // 1. Calculate Forces
                // Thrust (forward direction based on orientation)
                // Get the underlying vector from the unit quaternion's rotation
                // Dereference the result of the multiplication to get Vector3
                let forward_vector: Vector3<f32> = *(character.orientation * Vector3::z_axis()); // Assuming Z is forward
                let thrust_force: Vector3<f32> = forward_vector * character.throttle * 20.0; // Arbitrary thrust scaling

                // Drag (opposite to velocity)
                let drag_force: Vector3<f32> = -character.velocity * drag_coefficient;

                // Net force (assuming mass = 1 for simplicity)
                let net_force: Vector3<f32> = thrust_force + gravity + drag_force;

                // 2. Update Velocity
                let acceleration: Vector3<f32> = net_force; // Since mass = 1
                character.velocity += acceleration * FRAME_TIME;

                // 3. Update Position
                character.position += character.velocity * FRAME_TIME;

                // Prevent falling through a hypothetical ground plane at y=0
                if character.position.y < 0.0 {
                    character.position.y = 0.0;
                    // Zero out vertical velocity on collision
                    if character.velocity.y < 0.0 {
                        character.velocity.y = 0.0;
                    }
                    // Optional: Add some friction on ground contact
                    character.velocity.x *= 0.9;
                    character.velocity.z *= 0.9;
                }

                state_changed = true; // Assume physics always changes state for now
            }
            // --- End 3D Physics Update ---

            if state_changed {
                broadcast_state_update(&clients, &state_guard);
            }
        } // MutexGuard for game_state dropped here

        // Maintain target tick rate
        let elapsed = loop_start_time.elapsed();
        if elapsed < tick_duration {
            tokio::time::sleep(tick_duration - elapsed).await;
        } else {
            warn!("Game loop tick duration exceeded target: {:?}", elapsed);
        }
    }
}

// Public function to run the server
pub async fn run_server(addr: SocketAddr) {
    env_logger::builder().format_timestamp_micros().init(); // Ensure logger is initialized
    info!("Starting 1984 RPG Server (flight-rs base) on {}...", addr);

    // Initialize shared state
    let clients: Clients = Arc::new(Mutex::new(HashMap::new()));
    let game_state: SharedGameState = Arc::new(Mutex::new(GameState::new())); // Initialize RPG GameState

    // Start the game loop in a separate task
    let game_loop_clients = clients.clone();
    let game_loop_state = game_state.clone();
    tokio::spawn(async move {
        game_loop(game_loop_clients, game_loop_state).await;
    });

    // --- Define Warp Routes ---
    let ws_route = warp::path("ws")
        .and(warp::ws())
        .and(with_clients(clients.clone()))
        .and(with_game_state(game_state.clone()))
        .map(|ws: Ws, clients_map, game_state_map| {
            ws.on_upgrade(move |socket| handle_connection(socket, clients_map, game_state_map))
        });

    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("web/index.html"));

    // Combine routes, using warp::fs::dir directly
    let routes = ws_route.or(index).or(warp::fs::dir("web")); // Corrected route definition

    // Start the server
    info!("Listening for connections on http://{}", addr);
    warp::serve(routes).run(addr).await;
}

// Remove the misplaced module declarations from the end if they exist
// pub mod aircraft; // REMOVE
// pub mod input_state; // REMOVE
// pub mod physics; // REMOVE (already declared at top if needed)
// pub mod rpg_structs; // REMOVE (already declared at top)

// pub use aircraft::Aircraft; // REMOVE
// pub use input_state::InputState; // REMOVE
// pub use physics::PhysicsConfig; // REMOVE
// pub use rpg_structs::*; // REMOVE (already declared at top)
