"""
Test suite for the Flight Simulator server.

These tests ensure that the server can handle connections and manage aircraft state properly.
"""

import asyncio
import json
import os
import socket
import subprocess
import time
from pathlib import Path
from typing import AsyncGenerator, Dict, List, Optional, Tuple

import pytest
import websockets
from websockets.client import WebSocketClientProtocol

# Project root
PROJECT_ROOT = Path(__file__).parent.parent.parent

# Server process
SERVER_PROCESS: Optional[subprocess.Popen] = None
SERVER_PORT = 8081  # Use a different port than default to avoid conflicts


@pytest.fixture(scope="session", autouse=True)
async def server_setup():
    """Start the server before tests and stop it after."""
    # Start the server in a subprocess
    global SERVER_PROCESS
    
    # Check if server is already running on test port
    if is_port_in_use(SERVER_PORT):
        pytest.skip(f"Port {SERVER_PORT} is already in use")
        return
    
    # Start server process
    env = os.environ.copy()
    env["RUST_LOG"] = "info"
    
    cmd = ["cargo", "run", "--bin", "server", "--", f"--port={SERVER_PORT}"]
    SERVER_PROCESS = subprocess.Popen(
        cmd,
        cwd=str(PROJECT_ROOT),
        env=env,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    
    # Wait for server to start
    server_ready = False
    for _ in range(30):  # Wait up to 3 seconds
        if is_port_in_use(SERVER_PORT):
            server_ready = True
            break
        await asyncio.sleep(0.1)
    
    if not server_ready:
        # Read any output from the server to help diagnose the issue
        if SERVER_PROCESS.poll() is not None:  # Process has terminated
            stdout_data = SERVER_PROCESS.stdout.read().decode("utf-8") if SERVER_PROCESS.stdout else ""
            stderr_data = SERVER_PROCESS.stderr.read().decode("utf-8") if SERVER_PROCESS.stderr else ""
            pytest.fail(f"Server failed to start. Exit code: {SERVER_PROCESS.returncode}\n"
                       f"STDOUT: {stdout_data}\nSTDERR: {stderr_data}")
        else:
            pytest.fail(f"Server did not start listening on port {SERVER_PORT} within timeout")
    
    yield  # Run the tests
    
    # Terminate the server
    if SERVER_PROCESS and SERVER_PROCESS.poll() is None:
        SERVER_PROCESS.terminate()
        try:
            SERVER_PROCESS.wait(timeout=5)
        except subprocess.TimeoutExpired:
            SERVER_PROCESS.kill()


def is_port_in_use(port: int) -> bool:
    """Check if a port is in use."""
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        return s.connect_ex(('localhost', port)) == 0


@pytest.fixture
async def client() -> AsyncGenerator[WebSocketClientProtocol, None]:
    """Create a WebSocket client connected to the server."""
    async with websockets.connect(f"ws://localhost:{SERVER_PORT}/ws") as websocket:
        # Wait for welcome message
        welcome_msg = await websocket.recv()
        assert "Welcome!" in welcome_msg
        yield websocket


async def send_input(ws: WebSocketClientProtocol, 
                     pitch_up: bool = False, 
                     pitch_down: bool = False,
                     throttle_up: bool = False,
                     throttle_down: bool = False) -> None:
    """Send input to the server."""
    input_data = {
        "pitch_up": pitch_up,
        "pitch_down": pitch_down,
        "throttle_up": throttle_up,
        "throttle_down": throttle_down
    }
    await ws.send(json.dumps(input_data))


async def get_game_state(ws: WebSocketClientProtocol, timeout: float = 1.0) -> Dict:
    """Get the current game state from the server."""
    try:
        # Wait for state update
        state_json = await asyncio.wait_for(ws.recv(), timeout)
        state = json.loads(state_json)
        return state
    except asyncio.TimeoutError:
        pytest.fail("Timed out waiting for game state")


@pytest.mark.asyncio
async def test_connection(client: WebSocketClientProtocol):
    """Test that the client can connect to the server."""
    # Connection is established in the fixture
    assert client.open


@pytest.mark.asyncio
async def test_throttle_control(client: WebSocketClientProtocol):
    """Test that throttle control works."""
    # Get initial state
    initial_state = await get_game_state(client)
    
    # Find our aircraft
    our_aircraft = initial_state["aircraft"][0]
    initial_throttle = our_aircraft["throttle_level"]
    
    # Send throttle up command
    await send_input(client, throttle_up=True)
    
    # Wait for a few frames to ensure the input is processed
    await asyncio.sleep(0.2)
    
    # Get updated state
    updated_state = await get_game_state(client)
    
    # Find our aircraft again
    for aircraft in updated_state["aircraft"]:
        if aircraft["id"] == our_aircraft["id"]:
            # Throttle should have increased
            assert aircraft["throttle_level"] > initial_throttle
            break
    else:
        pytest.fail("Couldn't find our aircraft in updated state")


@pytest.mark.asyncio
async def test_pitch_control(client: WebSocketClientProtocol):
    """Test that pitch control works."""
    # Get initial state
    initial_state = await get_game_state(client)
    
    # Find our aircraft
    our_aircraft = initial_state["aircraft"][0]
    initial_theta = our_aircraft["theta"]
    
    # Send pitch up command
    await send_input(client, pitch_up=True)
    
    # Wait for a few frames
    await asyncio.sleep(0.2)
    
    # Get updated state
    updated_state = await get_game_state(client)
    
    # Find our aircraft again
    for aircraft in updated_state["aircraft"]:
        if aircraft["id"] == our_aircraft["id"]:
            # Pitch should have increased (become more positive)
            assert aircraft["theta"] > initial_theta
            break
    else:
        pytest.fail("Couldn't find our aircraft in updated state")


@pytest.mark.asyncio
async def test_multiple_clients():
    """Test that multiple clients can connect and control separate aircraft."""
    # Connect two clients
    async with websockets.connect(f"ws://localhost:{SERVER_PORT}/ws") as client1, \
               websockets.connect(f"ws://localhost:{SERVER_PORT}/ws") as client2:
        
        # Get welcome messages to extract IDs
        welcome1 = await client1.recv()
        welcome2 = await client2.recv()
        
        # Extract client IDs
        client1_id = welcome1.split(":")[1].strip()
        client2_id = welcome2.split(":")[1].strip()
        
        # Ensure different IDs
        assert client1_id != client2_id
        
        # Get initial states
        state1 = await get_game_state(client1)
        state2 = await get_game_state(client2)
        
        # Should have same number of aircraft (2)
        assert len(state1["aircraft"]) == 2
        assert len(state2["aircraft"]) == 2
        
        # Apply different controls to each client
        await send_input(client1, throttle_up=True)
        await send_input(client2, pitch_up=True)
        
        # Wait for inputs to take effect
        await asyncio.sleep(0.3)
        
        # Get updated states
        updated_state1 = await get_game_state(client1)
        updated_state2 = await get_game_state(client2)
        
        # Find aircraft by ID
        client1_aircraft = None
        client2_aircraft = None
        
        for aircraft in updated_state1["aircraft"]:
            if aircraft["id"] == client1_id:
                client1_aircraft = aircraft
            elif aircraft["id"] == client2_id:
                client2_aircraft = aircraft
        
        # Verify each aircraft has been affected by its controls
        assert client1_aircraft is not None
        assert client2_aircraft is not None
        
        # Client 1 should have higher throttle
        assert client1_aircraft["throttle_level"] > 0
        
        # Client 2 should have positive pitch
        assert client2_aircraft["theta"] > 0 