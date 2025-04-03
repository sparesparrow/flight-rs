console.log("1984 RPG Client Script Loaded");

// --- Configuration ---
const RECONNECT_DELAY = 3000; // Milliseconds

// --- State Variables ---
let socket = null;
let myPlayerId = null;
let currentGameState = null;

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
    currentGameState = newGameState;

    // If game content isn't visible yet, but we now have a character, show it.
    if (currentGameState.players[myPlayerId] && gameContentDiv.classList.contains('hidden')) {
        hideElement(characterCreationDiv);
        showElement(gameContentDiv);
    }

    // Update all UI elements based on the new state
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

// --- Initialisation ---
addLogEntry("Initializing client...");
connectWebSocket();
