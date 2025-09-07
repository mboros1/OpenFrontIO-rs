# OpenFront Rust Client

High-performance Rust implementation of the OpenFront.io game client, designed to significantly reduce CPU usage and improve battery life compared to the JavaScript implementation.

## Architecture Overview

This client reimplements the rendering and networking layers while maintaining compatibility with the existing server infrastructure.

### Key Components

- **Rendering**: Bevy ECS with wgpu backend for GPU-accelerated graphics
- **UI**: egui for immediate-mode overlays and menus  
- **Networking**: tokio-tungstenite for WebSocket communication
- **Serialization**: JSON for compatibility, with migration path to bincode

## Performance Improvements

Expected improvements over JavaScript client:
- **50-70%** reduction in CPU usage
- **3-5x** better battery life
- Consistent 60 FPS with thousands of units
- Significantly reduced thermal output

## Building

### Native Desktop Client
```bash
cargo build --release
```

### WASM Browser Client  
```bash
cargo build --release --target wasm32-unknown-unknown --features wasm
```

## Project Structure

```
src/
├── main.rs           # Entry point
├── networking/       # WebSocket transport layer
│   ├── transport.rs  # Message handling
│   └── schemas.rs    # Protocol definitions
├── rendering/        # Game rendering
│   ├── map.rs        # Terrain and territory
│   ├── units.rs      # Unit sprites
│   └── ui.rs         # UI overlays
├── game/             # Game state management
│   ├── state.rs      # Client-side game state
│   └── view.rs       # Game view model
└── input/            # Input handling
    └── handler.rs    # Mouse, keyboard, touch
```

## Development Status

- [x] Dependency research and selection
- [ ] Basic Bevy application setup
- [ ] WebSocket connection implementation
- [ ] Message protocol compatibility
- [ ] Basic rendering pipeline
- [ ] UI system integration
- [ ] Input handling
- [ ] Asset loading system
- [ ] Performance optimization
- [ ] WASM build support

## Migration from TypeScript Client

The Rust client maintains protocol compatibility with the existing server, allowing for:
1. Side-by-side testing with the TypeScript client
2. Gradual migration of features
3. A/B performance testing
4. Fallback to TypeScript client if needed

## Dependencies Rationale

- **Bevy**: Modern ECS architecture ideal for managing many game entities efficiently
- **egui**: Immediate mode UI reduces state management overhead
- **tokio-tungstenite**: Production-tested WebSocket implementation
- **wgpu**: Modern GPU API abstraction for optimal performance across platforms