use nalgebra::{Point3, UnitQuaternion, Vector3};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// --- New Structs for Cat Companion and Quest ---
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum CatStatus {
    Following,
    Waiting,
    Injured,
    Lost, // Maybe add more states later
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CatState {
    pub name: String,
    pub health: u8, // 0-100
    pub status: CatStatus,
    // We could add 3D position/orientation here too if the cat moves independently in 3D
    // pub position: Point3<f32>,
    // pub orientation: UnitQuaternion<f32>,
}
// --- End New Structs ---

/// Language of the forbidden text
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum TextLanguage {
    Czech,
    English,
}

/// Represents a fragment of forbidden anarcho-capitalist knowledge
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ForbiddenText {
    pub id: String,
    pub title: String,
    pub content: String,
    pub language: TextLanguage, // Czech or English
    pub difficulty: u8,         // 1-10 difficulty to understand
    pub suspicion_risk: u8,     // 1-10 risk of being caught with this text
}

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

    // --- Forbidden Knowledge State ---
    pub anarcho_knowledge: HashMap<String, u8>, // Topic -> Understanding level (0-100)
    pub economic_freedom_score: u8, // 0-100, affected by anarcho-capitalist understanding
    pub voluntary_actions: u32,     // Counter for voluntary exchanges/actions taken
    // --- End Forbidden Knowledge State ---

    // --- 3D Flight State ---
    pub position: Point3<f32>,
    pub velocity: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
    pub throttle: f32, // 0.0 to 1.0
    // --- End 3D Flight State ---

    // --- Cat Companion & Quest State ---
    pub cat_companion: Option<CatState>,
    pub kocourka_quest_active: bool,
    pub kocourka_quest_failed: bool,
    // --- End Cat Companion & Quest State ---
}

impl Character {
    // Basic constructor for a new character
    pub fn new(player_id: Uuid, name: String, occupation: String) -> Self {
        // Initialize base character
        let mut character = Character {
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

            // Initialize Forbidden Knowledge state
            anarcho_knowledge: HashMap::new(),
            economic_freedom_score: 0,
            voluntary_actions: 0,

            // Initialize 3D state
            position: Point3::new(0.0, 0.0, 1.7),
            velocity: Vector3::zeros(),
            orientation: UnitQuaternion::identity(),
            throttle: 0.0,

            // Initialize Cat & Quest state
            cat_companion: None, // Initially no cat
            kocourka_quest_active: false,
            kocourka_quest_failed: false,
        };

        // Initialize with empty anarcho-capitalist knowledge topics
        character
            .anarcho_knowledge
            .insert("Principles of Non-Aggression".to_string(), 0);
        character
            .anarcho_knowledge
            .insert("Voluntary Exchange".to_string(), 0);
        character
            .anarcho_knowledge
            .insert("Free Market Economy".to_string(), 0);
        character
            .anarcho_knowledge
            .insert("Private Property Rights".to_string(), 0);
        character
            .anarcho_knowledge
            .insert("Decentralization".to_string(), 0);

        // Assign the cat companion at creation
        character.cat_companion = Some(CatState {
            name: "Kocourek".to_string(), // Default cat name
            health: 100,
            status: CatStatus::Following,
            // Initialize position if needed
            // position: character.position, // Start near player
            // orientation: UnitQuaternion::identity(),
        });
        character.kocourka_quest_active = true; // Start quest immediately

        character
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

    // Add forbidden knowledge collection
    pub forbidden_texts: HashMap<String, ForbiddenText>, // id -> ForbiddenText
    pub text_locations: HashMap<String, Vec<String>>,    // location -> text_ids
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
        npcs.insert(
            "Old Trader".to_string(),
            Npc {
                name: "Old Trader".to_string(),
                description: "An elderly prole who trades in black market goods and seems to remember life before the Party."
                    .to_string(),
                trust: 70,
                location: "Prole District".to_string(),
            },
        );

        // Initialize forbidden knowledge texts
        let mut forbidden_texts = HashMap::new();
        let mut text_locations = HashMap::new();

        // Add anarcho-capitalist texts
        forbidden_texts.insert(
            "ankap_principles".to_string(),
            ForbiddenText {
                id: "ankap_principles".to_string(),
                title: "Principy dobrovolnosti".to_string(), // Principles of Voluntariness
                content: "Anarchokapitalismus je založen na myšlence oboustranné dobrovolnosti: Nikdo by neměl být nucen a nikomu by nemělo být bráněno nabízet ostatním produkty své práce za libovolných podmínek...".to_string(),
                language: TextLanguage::Czech,
                difficulty: 5,
                suspicion_risk: 8,
            },
        );

        forbidden_texts.insert(
            "free_market".to_string(),
            ForbiddenText {
                id: "free_market".to_string(),
                title: "Volný trh a svoboda jednotlivce".to_string(), // Free Market and Individual Freedom
                content: "Chceme jen svobodně žít v klidu a míru; chceme milovat, bavit se, pracovat, rozhodovat o sobě. Nechceme a nepotřebujeme nikoho, kdo si bude násilím brát plody naší práce...".to_string(),
                language: TextLanguage::Czech,
                difficulty: 6,
                suspicion_risk: 9,
            },
        );

        forbidden_texts.insert(
            "state_myth".to_string(),
            ForbiddenText {
                id: "state_myth".to_string(),
                title: "Mýtus nezbytnosti státu".to_string(), // The Myth of State Necessity
                content: "To je sice hezká pohádka, ale bez státu by naše společnost prostě nefungovala. Tak zněla má reakce, když jsem o anarchokapitalismu slyšel poprvé...".to_string(),
                language: TextLanguage::Czech,
                difficulty: 7,
                suspicion_risk: 10,
            },
        );

        // Hidden text in English for international understanding
        forbidden_texts.insert(
            "freedom_eng".to_string(),
            ForbiddenText {
                id: "freedom_eng".to_string(),
                title: "The Path to Freedom".to_string(),
                content: "We want only to live freely in peace; we want to love, have fun, work, and make our own decisions. We don't want or need anyone who would forcibly take the fruits of our labor...".to_string(),
                language: TextLanguage::English,
                difficulty: 3,
                suspicion_risk: 7,
            },
        );

        // Place texts in locations
        text_locations.insert(
            "Charrington's Shop".to_string(),
            vec!["ankap_principles".to_string(), "freedom_eng".to_string()],
        );
        text_locations.insert(
            "Prole District".to_string(),
            vec!["free_market".to_string()],
        );
        text_locations.insert(
            "Ministry of Truth".to_string(),
            vec!["state_myth".to_string()],
        );

        WorldState {
            locations,
            npcs,
            current_date: "April 4, 1984".to_string(),
            two_minutes_hate_today: true,
            chocolate_ration: 30, // grams
            current_enemy: "Eurasia".to_string(),
            forbidden_texts,
            text_locations,
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

    // --- Anarcho-Capitalist Mechanics Messages ---
    ForbiddenTextFound {
        texts: Vec<String>, // List of text IDs found at current location
    },
    ForbiddenTextContent {
        text: ForbiddenText,
        understanding_increase: u8,
        suspicion_increase: u8,
    },
    KnowledgeShared {
        success: bool,
        target_reaction: String,
        consequence: String,
    },
    TeleScreenWarning {
        message: String, // Warning about thoughtcrime
        severity: u8,    // 1-5 severity level
    },
    VoluntaryExchangeResult {
        success: bool,
        result_message: String,
        gained_item: Option<String>,
        lost_item: Option<String>,
    },
    // --- End Anarcho-Capitalist Mechanics Messages ---
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

    // --- Anarcho-Capitalist Mechanics Messages ---
    SearchForForbiddenTexts,
    ReadForbiddenText {
        text_id: String,
    },
    HideForbiddenText {
        text_id: String,
        hiding_place: String, // where to hide it (e.g., "under floorboard")
    },
    DestroyForbiddenText {
        text_id: String,
    },
    MemorizeForbiddenKnowledge {
        topic: String,     // Which concept to focus on
        time_invested: u8, // 1-10 hours
    },
    ShareForbiddenKnowledge {
        target_npc: String,
        knowledge_topic: String,
        approach: SharingApproach,
    },
    VoluntaryExchange {
        target_npc: String,
        offer: String,
        request: String,
    },
    DisableTelescreen {
        method: String, // How the player is attempting to disable surveillance
    },
    // --- End Anarcho-Capitalist Mechanics Messages ---
}

// --- Additional Anarcho-Capitalist types ---

/// Different approaches to sharing forbidden knowledge
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum SharingApproach {
    Subtle,      // Reduced chance of detection but less understanding
    Direct,      // Better understanding but higher risk
    Metaphoric,  // Uses allegories and references
    Questioning, // Socratic method, question Party dogma
}

/// Types of consequences for engaging with forbidden knowledge
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ThoughtcrimeConsequence {
    None,
    Suspicion {
        amount: u8,
    },
    Surveillance {
        duration_days: u8,
    },
    Interrogation {
        location: String,
        interrogator: String,
    },
    Arrest {
        reason: String,
    },
}
