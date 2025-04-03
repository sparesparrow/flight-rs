use nalgebra::{Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Represents a single player's character
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Character {
    pub player_id: Uuid,
    pub name: String,
    pub occupation: String,
    pub loyalty: u8,      // 0-100
    pub suspicion: u8,    // 0-100
    pub thoughtcrime: u8, // 0-100
    pub health: u8,       // 0-100
    pub inventory: Vec<String>,
    pub relationships: HashMap<String, i8>, // NPC name -> Trust level (-100 to 100)
    pub location: String,                   // Key into WorldState.locations (RPG location)
    pub journal_entries: Vec<String>,
    pub tasks_completed: u32,
    pub rebellion_score: u8, // 0-100

    // --- 3D Flight State ---
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub throttle: f32, // 0.0 to 1.0
                       // --- End 3D Flight State ---
}

impl Character {
    // Basic constructor for a new character
    pub fn new(player_id: Uuid, name: String, occupation: String) -> Self {
        Character {
            player_id,
            name,
            occupation,
            loyalty: 50,
            suspicion: 0,
            thoughtcrime: 0,
            health: 100,
            inventory: Vec::new(),
            relationships: HashMap::new(),
            location: String::from("Victory Mansions"), // Starting RPG location
            journal_entries: Vec::new(),
            tasks_completed: 0,
            rebellion_score: 0,
            // Initialize 3D state
            position: Point3::new(0.0, 0.0, 1.7), // Start on the ground (approx human height)
            velocity: Vector3::zeros(),
            orientation: UnitQuaternion::identity(),
            throttle: 0.0,
        }
    }
}

// Represents a location in the world
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Location {
    pub name: String,
    pub description: String,
    pub connections: Vec<String>, // Names of connected locations
    pub safety: u8,               // 1-5 scale (5 is safest)
}

// Represents a Non-Player Character
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Npc {
    pub name: String,
    pub description: String,
    pub trust: i8,        // Base trust/betrayal factor
    pub location: String, // Key into WorldState.locations
}

// Represents the static and dynamic state of the game world
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WorldState {
    pub locations: HashMap<String, Location>,
    pub npcs: HashMap<String, Npc>,
    pub current_date: String,
    pub two_minutes_hate_today: bool,
    pub chocolate_ration: u8,
    pub current_enemy: String, // "Eurasia" or "Eastasia"
}

impl WorldState {
    // Initialize the world with default 1984 settings
    pub fn initialize() -> Self {
        let mut locations = HashMap::new();
        let mut npcs = HashMap::new();

        // --- Define Locations ---
        locations.insert("Victory Mansions".to_string(), Location {
            name: "Victory Mansions".to_string(),
            description: "Your dilapidated apartment building. The telescreen on the wall continuously broadcasts Party propaganda.".to_string(),
            connections: vec!["Ministry of Truth".to_string(), "Victory Square".to_string()],
            safety: 3,
        });
        locations.insert("Ministry of Truth".to_string(), Location {
            name: "Ministry of Truth".to_string(),
            description: "A massive pyramidal structure where historical documents are rewritten to match Party narratives.".to_string(),
            connections: vec!["Victory Mansions".to_string(), "Victory Square".to_string(), "Canteen".to_string()],
            safety: 1,
        });
        locations.insert(
            "Canteen".to_string(),
            Location {
                name: "Canteen".to_string(),
                description: "A gray cafeteria serving tasteless Victory meals and Victory Gin."
                    .to_string(),
                connections: vec!["Ministry of Truth".to_string()],
                safety: 2,
            },
        );
        locations.insert(
            "Victory Square".to_string(),
            Location {
                name: "Victory Square".to_string(),
                description: "The central square where public executions and rallies are held."
                    .to_string(),
                connections: vec![
                    "Victory Mansions".to_string(),
                    "Ministry of Truth".to_string(),
                    "Prole District".to_string(),
                    "Charrington's Shop".to_string(),
                ],
                safety: 1,
            },
        );
        locations.insert(
            "Prole District".to_string(),
            Location {
                name: "Prole District".to_string(),
                description:
                    "The rundown area where the proles (working class) live with less surveillance."
                        .to_string(),
                connections: vec![
                    "Victory Square".to_string(),
                    "Charrington's Shop".to_string(),
                ],
                safety: 4,
            },
        );
        locations.insert("Charrington's Shop".to_string(), Location {
            name: "Charrington's Shop".to_string(),
            description: "An antique shop run by an elderly man. It has a room upstairs without a telescreen.".to_string(),
            connections: vec!["Victory Square".to_string(), "Prole District".to_string()],
            safety: 3,
        });
        locations.insert("Ministry of Love".to_string(), Location {
            name: "Ministry of Love".to_string(),
            description: "The terrifying windowless building where enemies of the Party are taken. Room 101 is inside.".to_string(),
            connections: vec![], // No escape
            safety: 0,
        });

        // --- Define NPCs ---
        npcs.insert(
            "O'Brien".to_string(),
            Npc {
                name: "O'Brien".to_string(),
                description:
                    "A high-ranking Inner Party member who seems to have rebellious tendencies."
                        .to_string(),
                trust: 0, // Will betray you
                location: "Ministry of Truth".to_string(),
            },
        );
        npcs.insert(
            "Julia".to_string(),
            Npc {
                name: "Julia".to_string(),
                description:
                    "A young woman who works in the Fiction Department of the Ministry of Truth."
                        .to_string(),
                trust: 80,
                location: "Ministry of Truth".to_string(),
            },
        );
        npcs.insert(
            "Charrington".to_string(),
            Npc {
                name: "Charrington".to_string(),
                description: "The seemingly friendly old man who runs the antique shop."
                    .to_string(),
                trust: -100, // Thought Police agent
                location: "Charrington's Shop".to_string(),
            },
        );
        npcs.insert(
            "Parsons".to_string(),
            Npc {
                name: "Parsons".to_string(),
                description:
                    "Your neighbor, an enthusiastic Party supporter whose children spy on adults."
                        .to_string(),
                trust: 20,
                location: "Victory Mansions".to_string(),
            },
        );
        npcs.insert(
            "Syme".to_string(),
            Npc {
                name: "Syme".to_string(),
                description:
                    "A philologist working on the 11th edition of the Newspeak dictionary."
                        .to_string(),
                trust: 50,
                location: "Canteen".to_string(),
            },
        );

        WorldState {
            locations,
            npcs,
            current_date: "April 4th, 1984".to_string(),
            two_minutes_hate_today: true,
            chocolate_ration: 20,
            current_enemy: "Eurasia".to_string(),
        }
    }
}

// Represents the overall state of the game, including all players
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GameState {
    pub players: HashMap<Uuid, Character>,
    pub world_state: WorldState,
    pub day: u32,
}

impl GameState {
    pub fn new() -> Self {
        GameState {
            players: HashMap::new(),
            world_state: WorldState::initialize(),
            day: 1,
        }
    }
}

// Enum for messages sent from Server to Client
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ServerMessage {
    Welcome {
        player_id: Uuid,
        initial_game_state: GameState,
    },
    PlayerJoined {
        player_id: Uuid,
        character: Character,
    },
    PlayerLeft {
        player_id: Uuid,
    },
    GameStateUpdate(GameState), // Send the whole state (can be optimized later)
    NarrativeUpdate(String),    // Text description of events
    Error(String),
}

// Enum for messages sent from Client to Server
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ClientMessage {
    RequestCharacterCreation {
        name: String,
        occupation: String,
    },
    MoveRequest {
        // For RPG map movement
        target_location: String,
    },
    FlyInput {
        // For 3D flight control
        pitch: f32,           // -1.0 to 1.0
        roll: f32,            // -1.0 to 1.0
        yaw: f32,             // -1.0 to 1.0
        throttle_change: f32, // -1.0 to 1.0 (change delta)
    },
    InteractRequest {
        npc_name: String,
        interaction_type: u8,
    }, // interaction_type maps to choices
    JournalWriteRequest {
        entry: String,
    },
    SearchRequest,
    WorkRequest,
    RestRequest,
    // Add other actions as needed
}
