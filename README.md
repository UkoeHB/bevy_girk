# Game instance framework {INITIAL RELEASE IS WIP}

A framework for instanced multiplayer games with Bevy. An instanced multiplayer game is one where the player list is known before starting a game, and game durations are relatively short (a few hours at most).

The framework includes:
- **Game/client app framework**: This is a thin layer around your game app and client app that interfaces with the network layer and manages initialization and reconnects.
- **Game instance**: A game instance encapsulates a running game app. The game instance crate contains various utilities for launching and managing game instances.
- **Server backend**
    - **Host server**: Manages lobbies and launches new games on connected game hub servers, with basic load balancing. Allows users to reconnect to games.
    - **Game hub server**: Creates and manages game instances and sends game-over reports to the host server, which are forwarded to game participants.
- **Wiring**: Convenience tools for setting up game and client apps.

The framework uses [`bevy_replicon`] for networking (on top of [`bevy_renet`]). Clients can be set up to work on both native and WASM targets (pending a `bevy_renet` WASM transport).

For a complete working example, see [`bevy_girk_demo`] (WIP).


### TODO

This project is currently only the starting point for a multiplayer game architecture. Future work involves greatly expanding the server backend to include user authentication, a user profile database, matchmaking, match history, rankings, auto-updates for native clients, etc.

There may be limits to how far this framework can go while remaining generic, so this project may need to stabilize at a partial/minimal solution.
