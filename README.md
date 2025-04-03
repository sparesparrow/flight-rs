# Flight Simulator - Multi-platform Client-Server Architecture

A sophisticated flight simulator demonstrating cross-platform development skills using multiple technologies:

[![Rust CI/CD](https://github.com/yourusername/flight-rs/actions/workflows/rust.yml/badge.svg)](https://github.com/yourusername/flight-rs/actions/workflows/rust.yml)
[![MicroPython Client](https://github.com/yourusername/flight-rs/actions/workflows/micropython.yml/badge.svg)](https://github.com/yourusername/flight-rs/actions/workflows/micropython.yml)

## Architecture Overview

```mermaid
graph TD
    subgraph rust["Rust Server (Physics Engine)"]
        physics["Physics Module"] --> server["WebSocket Server"]
        server --> state["Game State Manager"]
    end
    
    subgraph clients["Clients"]
        web["Web Browser Client"]
        micropython["MicroPython Client (ESP32/PyBoard)"]
        future["Future Clients..."]
    end
    
    server -- "Aircraft State (JSON)" --> web
    server -- "Aircraft State (JSON)" --> micropython
    server -- "Aircraft State (JSON)" --> future
    
    web -- "Control Inputs (JSON)" --> server
    micropython -- "Control Inputs (JSON)" --> server
    future -- "Control Inputs (JSON)" --> server
    
    subgraph tools["Development Tools"]
        docker["Docker Container"]
        poetry["Python Poetry"]
        ci["GitHub Actions CI/CD"]
    end
    
    style rust fill:#ddd,stroke:#000,stroke-width:2px
    style clients fill:#f9f9f9,stroke:#000,stroke-width:2px
    style tools fill:#f6f6f6,stroke:#000,stroke-width:2px
```

## Components

### Rust Server

The core of the flight simulator, implementing accurate flight physics and enabling real-time multiplayer experiences.

```mermaid
classDiagram
    class Aircraft {
        +float x
        +float y
        +float vx
        +float vy
        +float theta
        +float throttle_level
        +InputState input
        +new() Aircraft
        +update(float dt) void
    }
    
    class InputState {
        +bool pitch_up
        +bool pitch_down
        +bool throttle_up
        +bool throttle_down
    }
    
    class WebSocketServer {
        -HashMap~String, Aircraft~ aircraftMap
        -HashMap~String, WebSocketConnection~ clients
        +handleConnection(socket) void
        +broadcastState() void
        +gameLoop() void
    }
    
    WebSocketServer o-- Aircraft : manages
    Aircraft *-- InputState : contains
```

### Client Architecture

```mermaid
flowchart LR
    subgraph WebClient["Web Client"]
        WebUI["HTML/CSS UI"] --> WebsocketJS["WebSocket Client"]
        WebUI --> RenderingEngine["Canvas Rendering"]
        UserInput["Keyboard Input"] --> WebsocketJS
    end
    
    subgraph MicroClient["MicroPython Client"]
        MicroUI["OLED Display"] --> MicroWS["WebSocket Client"]
        Hardware["Hardware Input\n(Buttons/Joystick)"] --> MicroWS
        LED["Status LEDs"] --> MicroWS
    end
    
    WebsocketJS --> Server["Rust WebSocket\nServer"]
    MicroWS --> Server
    
    style WebClient fill:#f9f9f9,stroke:#000,stroke-width:1px
    style MicroClient fill:#e6f7ff,stroke:#000,stroke-width:1px
    style Server fill:#ddd,stroke:#000,stroke-width:2px
```

## Features

- Real-time flight simulation with accurate physics
- Cross-platform multiplayer capability
- Embedded device support via MicroPython
- Comprehensive testing infrastructure
- Containerized development and testing

## Development Environment

```mermaid
graph TD
    subgraph Local["Local Development"]
        IDE["VS Code/Cursor"] --> RustDev["Rust Development"]
        IDE --> PythonDev["Python Development"]
        IDE --> WebDev["Web Development"]
        Docker["Docker/Compose"] --> Container["Containerized Testing"]
    end
    
    subgraph CI["Continuous Integration"]
        Actions["GitHub Actions"] --> RustCI["Rust CI/CD"]
        Actions --> PythonCI["Python Testing"]
        Actions --> DockerCI["Docker Build/Test"]
    end
    
    subgraph Remote["Remote Testing"]
        SSH["SSH/Port Forwarding"] --> ESP["ESP32 Testing"]
        SSH --> PyBoard["PyBoard Testing"]
    end
    
    style Local fill:#f9f9f9,stroke:#000,stroke-width:1px
    style CI fill:#e6f7ff,stroke:#000,stroke-width:1px 
    style Remote fill:#ddd,stroke:#000,stroke-width:1px
```

## Data Flow

```mermaid
sequenceDiagram
    participant Client as Client (Web/Embedded)
    participant Server as Rust Server
    participant Physics as Physics Engine
    
    Client->>Server: Connect via WebSocket
    Server->>Client: Welcome message with ID
    Server->>Physics: Create new aircraft
    
    loop Game Loop
        Client->>Server: Send control input
        Server->>Physics: Update aircraft state
        Physics->>Server: Return new state
        Server->>Client: Broadcast game state
        Client->>Client: Update display/UI
    end
    
    Client->>Server: Disconnect
    Server->>Physics: Remove aircraft
```

## Getting Started

1. Start the Rust server: `cargo run --bin server`
2. Connect via web browser: http://localhost:8080
3. For MicroPython client, flash `micropython/client.py` to your device

### Using Docker

```bash
# Start the server
docker compose up server

# Run tests
docker compose up test

# Development environment
docker compose up dev
```

## Project Structure

```
flight-rs/
├── src/               # Rust server code
├── web/               # Web client
├── micropython/       # MicroPython client for embedded devices
│   └── lib/           # MicroPython libraries
├── python/            # Python development tools
│   ├── tests/         # pytest test suite
│   └── scripts/       # Development utilities
├── docker/            # Containerization for testing
└── .github/workflows/ # CI/CD pipelines
```

## Continuous Integration

This project uses GitHub Actions for continuous integration and testing. The workflows include:

- Building and testing the Rust server
- Linting and testing Python code
- Testing MicroPython client
- Building and testing Docker containers

## Contributing

1. Fork the repository
2. Create your feature branch: `git checkout -b feature/amazing-feature`
3. Commit your changes: `git commit -m 'Add some amazing feature'`
4. Push to the branch: `git push origin feature/amazing-feature`
5. Open a Pull Request
