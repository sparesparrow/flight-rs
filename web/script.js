import * as THREE from 'three';

console.log("1984 RPG Client Script Loaded with Three.js support");

// --- Configuration ---
const RECONNECT_DELAY = 3000; // Milliseconds

// --- State Variables ---
let socket = null;
let myPlayerId = null;
let currentGameState = null;
let playerInput = { pitch: 0, roll: 0, yaw: 0, throttle_change: 0 }; // Added input state
const keysPressed = {}; // Track currently pressed keys

// --- Three.js Variables ---
let scene, camera, renderer;
let groundPlane;
const players3D = {}; // Map player ID to their 3D object { mesh: THREE.Mesh, lastUpdate: timestamp }

// --- DOM Element References ---
const connectionStatus = document.getElementById('connection-status');
const playerIdDisplay = document.getElementById('player-id');
const characterCreationDiv = document.getElementById('character-creation');
const gameContentDiv = document.getElementById('game-content');
const charNameInput = document.getElementById('char-name');
const charOccupationSelect = document.getElementById('char-occupation');
const createCharButton = document.getElementById('create-char-button');

// Game Content Panels
const characterStatsPanel = document.getElementById('character-stats');
const locationInfoPanel = document.getElementById('location-info');
const narrativeLogPanel = document.getElementById('narrative-log');
const journalPanel = document.getElementById('journal');
const inventoryPanel = document.getElementById('inventory');
const playerListPanel = document.getElementById('player-list');

// Specific UI Elements within panels
const statName = document.getElementById('stat-name');
const statOccupation = document.getElementById('stat-occupation');
const statHealth = document.getElementById('stat-health');
const statLoyalty = document.getElementById('stat-loyalty');
const statSuspicion = document.getElementById('stat-suspicion');
const statThoughtcrime = document.getElementById('stat-thoughtcrime');
const statRebellion = document.getElementById('stat-rebellion');

const locationName = document.getElementById('location-name');
const locationDescription = document.getElementById('location-description');
const locationSafety = document.getElementById('location-safety');
const moveOptionsDiv = document.getElementById('move-options');
const npcOptionsDiv = document.getElementById('npc-options');

const logEntriesDiv = document.getElementById('log-entries');
const journalDisplay = document.getElementById('journal-display');
const journalEntryInput = document.getElementById('journal-entry');
const journalSubmitButton = document.getElementById('journal-submit');

const inventoryList = document.getElementById('inventory-list');
const presentPlayersList = document.getElementById('present-players-list');

// Action Buttons
const actionSearchButton = document.getElementById('action-search');
const actionWorkButton = document.getElementById('action-work');
const actionRestButton = document.getElementById('action-rest');

const threeJsContainer = document.getElementById('threejs-container'); // Get the container

// --- Utility Functions ---
function showElement(element) {
    element?.classList.remove('hidden');
}

function hideElement(element) {
    element?.classList.add('hidden');
}

function addLogEntry(text, type = 'normal') {
    const entry = document.createElement('p');
    entry.textContent = text;
    entry.classList.add(`log-${type}`); // For potential styling (e.g., log-error, log-event)

    // Prepend so newest messages are at the top
    logEntriesDiv.insertBefore(entry, logEntriesDiv.firstChild);

    // Limit log size (optional)
    // while (logEntriesDiv.children.length > 100) {
    //     logEntriesDiv.removeChild(logEntriesDiv.lastChild);
    // }
}

// --- Three.js Functions ---
function initThreeJS() {
    console.log("Initializing Three.js scene...");

    // 1. Scene
    scene = new THREE.Scene();
    scene.background = new THREE.Color(0x222222); // Match body background
    scene.fog = new THREE.Fog(0x222222, 10, 100); // Add some fog for depth

    // 2. Camera
    const fov = 75;
    const aspect = window.innerWidth / window.innerHeight;
    const near = 0.1;
    const far = 200; // Increased far plane
    camera = new THREE.PerspectiveCamera(fov, aspect, near, far);
    camera.position.set(0, 5, 10); // Position camera slightly up and back
    camera.lookAt(0, 0, 0);

    // 3. Renderer
    renderer = new THREE.WebGLRenderer({ antialias: true });
    renderer.setSize(window.innerWidth, window.innerHeight);
    renderer.setPixelRatio(window.devicePixelRatio);
    threeJsContainer.appendChild(renderer.domElement);

    // 4. Lighting
    const ambientLight = new THREE.AmbientLight(0xaaaaaa); // Soft ambient light
    scene.add(ambientLight);

    const directionalLight = new THREE.DirectionalLight(0xffffff, 1.0);
    directionalLight.position.set(5, 10, 7.5);
    scene.add(directionalLight);
    // TODO: Add shadow support later if needed

    // 5. Ground Plane
    const groundGeometry = new THREE.PlaneGeometry(200, 200);
    const groundMaterial = new THREE.MeshStandardMaterial({ color: 0x555555, side: THREE.DoubleSide });
    groundPlane = new THREE.Mesh(groundGeometry, groundMaterial);
    groundPlane.rotation.x = -Math.PI / 2; // Rotate to be horizontal
    groundPlane.position.y = -0.1; // Slightly below origin
    scene.add(groundPlane);

    // Handle window resize
    window.addEventListener('resize', onWindowResize, false);

    console.log("Three.js scene initialized.");
}

function onWindowResize() {
    camera.aspect = window.innerWidth / window.innerHeight;
    camera.updateProjectionMatrix();
    renderer.setSize(window.innerWidth, window.innerHeight);
}

function animate() {
    requestAnimationFrame(animate);

    // --- Update Player Objects --- 
    updatePlayerObjects();

    // --- Update Input --- 
    handleInput(); // Process held keys and potentially send FlyInput

    // --- Render --- 
    if (renderer && scene && camera) {
        renderer.render(scene, camera);
    }
}

// --- WebSocket Functions ---
function connectWebSocket() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    // Use current host, assume server runs on same machine for default access
    const host = window.location.hostname || 'localhost';
    const port = window.location.port || '8080'; // Use the default port if not specified
    const wsUrl = `${protocol}//${host}:${port}/ws`;

    connectionStatus.textContent = `Connecting to ${wsUrl}...`;
    connectionStatus.style.color = '#ff0'; // Yellow
    console.log(`Attempting to connect to WebSocket: ${wsUrl}`);

    socket = new WebSocket(wsUrl);

    socket.onopen = () => {
        console.log("WebSocket connected");
        connectionStatus.textContent = 'Connected to Party Network';
        connectionStatus.style.color = '#0f0'; // Green
    };

    socket.onclose = (event) => {
        console.log("WebSocket disconnected", event);
        connectionStatus.textContent = `Disconnected from Party Network (Code: ${event.code}). Reconnecting...`;
        connectionStatus.style.color = '#f00'; // Red
        myPlayerId = null;
        currentGameState = null;
        playerIdDisplay.textContent = 'Your ID: N/A';
        hideElement(gameContentDiv);
        hideElement(characterCreationDiv);
        // Attempt to reconnect
        setTimeout(connectWebSocket, RECONNECT_DELAY);
    };

    socket.onerror = (error) => {
        console.error('WebSocket error:', error);
        connectionStatus.textContent = 'Error connecting to Party Network';
        connectionStatus.style.color = '#f00';
        // The onclose event will likely fire after this, triggering reconnect
    };

    socket.onmessage = (event) => {
        try {
            const serverMessage = JSON.parse(event.data);
            console.log("Received message:", serverMessage);
            handleServerMessage(serverMessage);
        } catch (e) {
            console.error('Failed to parse server message:', event.data, e);
            addLogEntry(`Error: Received unparseable message from server: ${event.data}`, 'error');
        }
    };
}

function sendMessage(clientMessage) {
    if (socket && socket.readyState === WebSocket.OPEN) {
        try {
            const messageString = JSON.stringify(clientMessage);
            console.log("Sending message:", clientMessage);
            socket.send(messageString);
        } catch (e) {
            console.error("Failed to serialize client message:", clientMessage, e);
            addLogEntry("Error: Could not send message to server.", 'error');
        }
    } else {
        console.warn("WebSocket not open. Cannot send message:", clientMessage);
        addLogEntry("Error: Not connected to server.", 'error');
    }
}

// --- Message Handling ---
function handleServerMessage(msg) {
    switch (Object.keys(msg)[0]) { // Check the type by the first key
        case 'Welcome':
            handleWelcome(msg.Welcome);
            break;
        case 'PlayerJoined':
            handlePlayerJoined(msg.PlayerJoined);
            break;
        case 'PlayerLeft':
            handlePlayerLeft(msg.PlayerLeft);
            break;
        case 'GameStateUpdate':
            handleGameStateUpdate(msg.GameStateUpdate);
            break;
        case 'NarrativeUpdate':
            handleNarrativeUpdate(msg.NarrativeUpdate);
            break;
        case 'Error':
            handleError(msg.Error);
            break;
        default:
            console.warn("Received unknown message type:", msg);
            addLogEntry(`Warning: Received unknown message type from server.`, 'warning');
    }
}

function handleWelcome(data) {
    myPlayerId = data.player_id;
    playerIdDisplay.textContent = `Your ID: ${myPlayerId}`;
    addLogEntry(`Connected to Party Network. Assigned ID: ${myPlayerId}`);

    // Update state immediately with the initial snapshot
    handleGameStateUpdate(data.initial_game_state);

    // Check if *this* player already has a character in the initial state
    const myCharacter = data.initial_game_state.players[myPlayerId];

    if (myCharacter) {
        console.log("Rejoined game, character already exists.");
        hideElement(characterCreationDiv);
        showElement(gameContentDiv);
        // Update UI based on existing character immediately
        updateUI(data.initial_game_state);
    } else {
        console.log("New player, showing character creation.");
        // Need to create character
        hideElement(gameContentDiv);
        showElement(characterCreationDiv);
    }
}

function handlePlayerJoined(data) {
    addLogEntry(`Party Member ${data.character.name} (${data.player_id.substring(0, 6)}...) joined.`);
    // We get full state updates, so we don't *need* to add them manually here,
    // but we could update the player list specifically if needed.
    // The next GameStateUpdate will include them.
}

function handlePlayerLeft(data) {
    addLogEntry(`Party Member ${data.player_id.substring(0, 6)}... disconnected.`);
    // We get full state updates, so removal will happen naturally.
    // Could update player list specifically here.
}

function handleGameStateUpdate(newGameState) {
    console.log("Updating game state:", newGameState);
    const previousGameState = currentGameState;
    currentGameState = newGameState;

    // If game content isn't visible yet, but we now have a character, show it.
    if (currentGameState.players[myPlayerId] && gameContentDiv.classList.contains('hidden')) {
        hideElement(characterCreationDiv);
        showElement(gameContentDiv);
    }

    // --- Update 3D Objects based on GameState --- 
    const activePlayerIds = new Set(Object.keys(currentGameState.players));

    // Add/Update players present in the new state
    for (const playerId in currentGameState.players) {
        const playerData = currentGameState.players[playerId];

        if (!players3D[playerId]) {
            // Player doesn't exist yet, create a 3D object
            console.log(`Creating 3D object for player ${playerId}`);
            const geometry = new THREE.BoxGeometry(1, 1, 2); // Simple box for now
            const material = new THREE.MeshStandardMaterial({
                color: playerId === myPlayerId ? 0x00ff00 : 0xff0000 // Green for self, red for others
            });
            const playerMesh = new THREE.Mesh(geometry, material);
            scene.add(playerMesh);
            players3D[playerId] = { mesh: playerMesh, lastUpdate: Date.now() };
        }

        // Update position and orientation
        const playerObj = players3D[playerId];
        if (playerData.position && playerObj) {
            playerObj.mesh.position.set(playerData.position.x, playerData.position.y, playerData.position.z);
        }
        if (playerData.orientation && playerObj) {
            // Nalgebra Quaternions are likely [x, y, z, w], Three.js uses (x, y, z, w)
            playerObj.mesh.quaternion.set(
                playerData.orientation.coords.x,
                playerData.orientation.coords.y,
                playerData.orientation.coords.z,
                playerData.orientation.coords.w
            );
        }
        playerObj.lastUpdate = Date.now(); // Mark as updated
    }

    // Remove players that are no longer in the game state
    for (const existingPlayerId in players3D) {
        if (!activePlayerIds.has(existingPlayerId)) {
            console.log(`Removing 3D object for disconnected player ${existingPlayerId}`);
            scene.remove(players3D[existingPlayerId].mesh);
            players3D[existingPlayerId].mesh.geometry.dispose(); // Clean up geometry
            players3D[existingPlayerId].mesh.material.dispose(); // Clean up material
            delete players3D[existingPlayerId];
        }
    }

    // Update the 2D UI elements (existing logic)
    updateUI(currentGameState);
}

function handleNarrativeUpdate(text) {
    addLogEntry(text, 'narrative');
}

function handleError(errorText) {
    addLogEntry(`BIG BROTHER SAYS: ${errorText}`, 'error');
    // Could also use alert() for critical errors
    // alert(`Server Error: ${errorText}`);
}

// --- UI Update Functions ---
function updateUI(state) {
    if (!state || !myPlayerId) return; // Can't update if no state or ID

    const myCharacter = state.players[myPlayerId];

    if (!myCharacter) {
        // Player exists in state but this client doesn't have a character yet?
        // Or player was removed (died/arrested)
        console.log("My character data not found in game state. Hiding game content.");
        hideElement(gameContentDiv);
        // Might need character creation again if they died and can rejoin?
        // Or show a "Game Over" screen.
        return;
    }

    // Update Character Stats Panel
    updateCharacterStatsUI(myCharacter);

    // Update Location Panel
    updateLocationUI(state.world_state, myCharacter);

    // Update Journal Panel
    updateJournalUI(myCharacter);

    // Update Inventory Panel
    updateInventoryUI(myCharacter);

    // Update Player List Panel
    updatePlayerListUI(state.players, myCharacter.location);
}

function updateCharacterStatsUI(character) {
    statName.textContent = character.name || 'N/A';
    statOccupation.textContent = character.occupation || 'N/A';
    statHealth.textContent = character.health ?? '??';
    statLoyalty.textContent = character.loyalty ?? '??';
    statSuspicion.textContent = character.suspicion ?? '??';
    statThoughtcrime.textContent = character.thoughtcrime ?? '??';
    statRebellion.textContent = character.rebellion_score ?? '??';
}

function updateLocationUI(worldState, myCharacter) {
    const currentLocName = myCharacter.location;
    const locationData = worldState.locations[currentLocName];

    if (!locationData) {
        locationName.textContent = 'Unknown';
        locationDescription.textContent = 'Lost in the ministries...';
        locationSafety.textContent = '';
        moveOptionsDiv.innerHTML = ''; // Clear options
        npcOptionsDiv.innerHTML = ''; // Clear options
        return;
    }

    locationName.textContent = locationData.name;
    locationDescription.textContent = locationData.description;
    locationSafety.textContent = `(Safety Level: ${locationData.safety}/5)`;

    // Update Move Options
    moveOptionsDiv.innerHTML = ''; // Clear previous options
    locationData.connections.forEach(connName => {
        const button = document.createElement('button');
        button.textContent = connName;
        button.onclick = () => sendMoveRequest(connName);
        moveOptionsDiv.appendChild(button);
    });

    // Update NPC Interaction Options
    npcOptionsDiv.innerHTML = ''; // Clear previous options
    Object.entries(worldState.npcs)
        .filter(([_, npcData]) => npcData.location === currentLocName) // Only NPCs at current location
        .forEach(([npcName, npcData]) => {
            const button = document.createElement('button');
            button.textContent = npcName;
            // TODO: Implement more complex interaction system later
            // For now, just a basic interaction request
            button.onclick = () => sendInteractRequest(npcName, 1); // Using 1 as placeholder type
            npcOptionsDiv.appendChild(button);
        });
}

function updateJournalUI(character) {
    journalDisplay.value = character.journal_entries.join('\n\n---\n\n');
    // Scroll to bottom
    journalDisplay.scrollTop = journalDisplay.scrollHeight;
}

function updateInventoryUI(character) {
    inventoryList.innerHTML = ''; // Clear previous items
    if (character.inventory.length === 0) {
        const li = document.createElement('li');
        li.textContent = '(Empty)';
        inventoryList.appendChild(li);
    } else {
        character.inventory.forEach(item => {
            const li = document.createElement('li');
            li.textContent = item;
            inventoryList.appendChild(li);
        });
    }
}

function updatePlayerListUI(allPlayers, myLocation) {
    presentPlayersList.innerHTML = ''; // Clear previous list
    let playersFound = false;
    Object.entries(allPlayers)
        .filter(([id, char]) => id !== myPlayerId && char.location === myLocation) // Others at my location
        .forEach(([id, char]) => {
            playersFound = true;
            const li = document.createElement('li');
            li.textContent = `${char.name} (${char.occupation}) - ${id.substring(0, 6)}...`;
            presentPlayersList.appendChild(li);
        });

    if (!playersFound) {
        const li = document.createElement('li');
        li.textContent = '(None)';
        presentPlayersList.appendChild(li);
    }
}


// --- Action Sending Functions ---
function sendCharacterCreation() {
    const name = charNameInput.value.trim();
    const occupation = charOccupationSelect.value;
    if (!name) {
        addLogEntry("Error: Please enter a name.", 'error');
        return;
    }
    sendMessage({ RequestCharacterCreation: { name, occupation } });
}

function sendMoveRequest(targetLocation) {
    sendMessage({ MoveRequest: { target_location: targetLocation } });
}

function sendInteractRequest(npcName, interactionType) {
    sendMessage({ InteractRequest: { npc_name: npcName, interaction_type: interactionType } });
}

function sendJournalWrite() {
    const entry = journalEntryInput.value.trim();
    if (!entry) {
        addLogEntry("Error: Journal entry cannot be empty.", 'error');
        return;
    }
    sendMessage({ JournalWriteRequest: { entry } });
    journalEntryInput.value = ''; // Clear input after sending
}

function sendSearchRequest() {
    sendMessage({ SearchRequest: {} });
}

function sendWorkRequest() {
    sendMessage({ WorkRequest: {} });
}

function sendRestRequest() {
    sendMessage({ RestRequest: {} });
}

// --- Input Handling --- 
function setupInputListeners() {
    window.addEventListener('keydown', (event) => {
        keysPressed[event.code] = true;
    });
    window.addEventListener('keyup', (event) => {
        keysPressed[event.code] = false;
    });
    // Could add mouse listeners here too if needed
}

function handleInput() {
    // Reset input changes for this frame
    playerInput.pitch = 0;
    playerInput.roll = 0;
    playerInput.yaw = 0;
    playerInput.throttle_change = 0;
    let inputChanged = false;

    // Map keys to inputs (adjust sensitivity/scaling as needed)
    if (keysPressed['KeyW'] || keysPressed['ArrowUp']) { playerInput.pitch = 1.0; inputChanged = true; }
    if (keysPressed['KeyS'] || keysPressed['ArrowDown']) { playerInput.pitch = -1.0; inputChanged = true; }
    if (keysPressed['KeyA'] || keysPressed['ArrowLeft']) { playerInput.roll = -1.0; inputChanged = true; } // Roll left
    if (keysPressed['KeyD'] || keysPressed['ArrowRight']) { playerInput.roll = 1.0; inputChanged = true; } // Roll right
    if (keysPressed['KeyQ']) { playerInput.yaw = -1.0; inputChanged = true; } // Yaw left
    if (keysPressed['KeyE']) { playerInput.yaw = 1.0; inputChanged = true; } // Yaw right
    if (keysPressed['ShiftLeft'] || keysPressed['ShiftRight']) { playerInput.throttle_change = 1.0; inputChanged = true; } // Increase throttle
    if (keysPressed['ControlLeft'] || keysPressed['ControlRight']) { playerInput.throttle_change = -1.0; inputChanged = true; } // Decrease throttle

    // Send FlyInput message if any input is active
    if (inputChanged) {
        sendMessage({ FlyInput: playerInput });
    }
}

// --- Event Listeners ---
createCharButton.addEventListener('click', sendCharacterCreation);
journalSubmitButton.addEventListener('click', sendJournalWrite);
actionSearchButton.addEventListener('click', sendSearchRequest);
actionWorkButton.addEventListener('click', sendWorkRequest);
actionRestButton.addEventListener('click', sendRestRequest);

// Add listener for Enter key on journal input
journalEntryInput.addEventListener('keypress', function (e) {
    if (e.key === 'Enter') {
        e.preventDefault(); // Prevent default form submission/newline
        sendJournalWrite();
    }
});

// --- Initialization ---
function init() {
    console.log("Initializing Client...");
    initThreeJS();      // Initialize Three.js first
    setupInputListeners(); // Setup keyboard listeners
    animate();          // Start the render loop
    connectWebSocket(); // Connect WebSocket

    // Add event listeners for existing UI
    createCharButton?.addEventListener('click', sendCharacterCreation);
    journalSubmitButton?.addEventListener('click', sendJournalWrite);
    actionSearchButton?.addEventListener('click', sendSearchRequest);
    actionWorkButton?.addEventListener('click', sendWorkRequest);
    actionRestButton?.addEventListener('click', sendRestRequest);

    // Event delegation for dynamic buttons (move, interact)
    locationInfoPanel?.addEventListener('click', (event) => {
        if (event.target.matches('.move-button')) {
            sendMoveRequest(event.target.dataset.location);
        }
        if (event.target.matches('.interact-button')) {
            // TODO: Implement interaction types if needed
            sendInteractRequest(event.target.dataset.npc, 0);
        }
    });

    console.log("Client Initialized.");
}

// Start the application
init();
