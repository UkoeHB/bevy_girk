# Usage

This document contains a detailed description of how to use `bevy_girk` to implement a game.

For the library architecture, see `ARCHITECTURE.md`.

To navigate the types and functions in this document it is recommended to open it in a text editor and use 'go to definition'.


---


## Overview

`bevy_girk` is designed so you can focus on writing game-specific logic. Networking and setup details are hidden by helper functions and types.

Since the backend needs to manage and launch your games, we use dependency injection to integrate your game-specific logic with `bevy_girk` servers and the game networking layer.

A project using `bevy_girk` will end up with a number of distinct apps. Each of those apps may or may not live in separate binaries (more on that below).

- **Game App**: A headless app with your core game logic (i.e. a 'single-threaded authoritative game server').
- **Client App**: A GUI app connected to a game app, this is where users play your game.
- **Backend**:
    - **Host Server**: A headless server app that manages connected users and lobbies.
    - **Game Hub Server**: A headless server app that receives game requests from the host server and runs game apps.
- **User Client**: A GUI app that connects to, and interacts with, the host server (making/joining/starting lobbies). The app receives game start packages from the host server and launches client apps. It is also helps coordinate client app reconnects for ongoing games.


---


## Wiring

Dependency injection ties your game-specific logic to the `bevy_girk` servers and networking layer. A `bevy_girk` project needs to implement a number of trait objects and define how to serialize/deserialize various data objects.


### Host Server

- **`LobbyChecker`** (trait object): Validates `LobbyData`, checks if new members can join a lobby, and determines whether a lobby can be launched as a game.
- **`LobbyData`** (data object): Produced by a user who creates a lobby, validated by `LobbyChecker` and consumed by `GameLaunchPackSource` for producing `GameLaunchPacks`. Sent to users by the host server during lobby setup so it can be used to display up-to-date lobby information (e.g. as members join/leave).
    - The custom data field in this type should deserialize to your custom lobby data.


### Game Hub Server

- **`GameLaunchPackSource`** (trait object): Converts `GameStartRequest`s (containing `LobbyData`s) into `GameLaunchPack`s with an asynchronous API. This allows you to insert extra data into launch packs beyond just the lobby data (e.g. based on a user database query to get user loadouts).
- **`GameLaunchPack`** (data object): Produced by a `GameLaunchPackSource`, used for game app setup by `GameFactory`. 
    - The game init data field in this type should deserialize to game-specific initialization details for a game.
    - **`ClientInitDataForGame`** (data object): Internal per-client data in a launch pack. The data field in this type should deserialize to per-client game-specific initialization details for a game. (TODO: consolidate data in launch packs to one blob)
- **`GameInstanceLauncher`** (trait object): Launches a game app, returns a `GameInstance` for managing the game. Two default implementations are provided:
    - `GameInstanceLauncherProcess`: Launches a game app binary in a child process.
        - Use `inprocess_game_launcher()` inside the binary. This helper uses a `GameFactory` to create your game app.
    - `GameInstanceLauncherLocal`: Launches a game app in a `std::thread` using a `GameFactory`.


### Game App

- **`GameFactory`** (trait object): Consumes `GameLaunchPack`s to set up game apps and produces `GameStartReport`s.
- **`GameStartReport`** (data object): Produced by a `GameFactory`, used to orchestrate client setup by the game manager (for multiplayer games this is the host server).
    - Includes auth info for creating `ServerConnectToken`s, which are used by client apps to connect to the game app's `renet` server. Start reports include auth info instead of connect tokens so that tokens can be produced on-demand by the host server to support reconnects.
    - **`GameStartInfo`** (data object): Per-client custom data, used for client setup by `ClientFactory`. The start data field in this type should deserialize to game-specific initialization details for a client app.
- **`GameOverReport`** (data object): A report that is submitted by custom game logic to the `GameEndFlag` resource. It will be extracted by systems inserted by `game_instance_setup()`. For multiplayer games, the report will ultimately be forward to the user client in a `HostToUserMsg::GameOver` message. The game over data field in this type should deserialize to a game-specific game over report.
- **`ClientRequestHandler`** (trait object): Bevy resource inserted into game apps and used to handle incoming game-specific client requests.
- **`GameMessageBuffer`**: Bevy resource inserted into game apps and used to marshal game messages into the networking backend.
    - `GameMessage`: Type specified on construction, must equal the type for all game network messages to be sent to the client (it should probably be a big enum).
    - *Note*: We use type id consistency checks in the `GameMessageBuffer` instead of making it generic so the game framework doesn't need to have generic functions and types everywhere.


### Client App

- **`ClientFactory`** (trait object): Uses `ServerConnectToken` and `GameStartInfo` to set up client apps.
- **`GameMessageHandler`** (trait object): Bevy resource inserted into client apps and used to handle incoming game-specific game messages.
- **`ClientRequestBuffer`**: Bevy resource inserted into client apps and used to marshal client requests into the networking backend.
    - `ClientRequest`: Type specified on construction, must equal the type for all client network requests to be sent to the game (it should probably be a big enum).
    - *Note*: We use type id consistency checks in the `ClientRequestBuffer` instead of making it generic so the client framework doesn't need to have generic functions and types everywhere.


---


## Implementing Apps

Your game infrastructure will be implemented in a set of Bevy apps.


### Backend

**Host Server**

- Use `make_host_server()` to make a host server app.
- See `HostToUserMsg`, `HostToUserResponse`, `UserToHostMsg`, `UserToHostRequest` for the host-user API.

**Game Hub Server**

- Use `make_game_hub_server()` to make a game hub server app.
- You can control the game hub by sending it `GameHubCommand`s.
- Multiple game hubs can be connected to the same host server, which will do basic load balancing based on the available capacity of connected game hubs.


### Game App

**Setup**

- Implement a `GameFactory` for making game apps. Your factory can use `prepare_girk_game_app()` to set up `bevy_girk`-related systems and resources.

**Replication**

- Use [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair) to register replicated components. No components are registered by default.

**App Startup**

- Insert a `ClientRequestHandler` resource to the app with your desired request-handling callback.
    - The callback can use `deserialize_client_request()?` to extract requests.
- Insert a `GameMessageBuffer` resource to the app with your desired game message type. The message type must implement `IntoEventType` for determining the message send policy (ordered/unordered/unreliable).

**API**

The game framework exposes a small API for your game logic.

- **`GameFwTicksElapsed`**: Bevy resource that counts how many game ticks have elapsed since startup. See the `GameFwTickPlugin` code docs for more details.
- **`GameInitProgress`**: Bevy component on an entity spawned by the framework at startup. It tracks the total initialization progress of all clients while the game is initializing. The entity includes `bevy_replicon::prelude::Replication` by default, so if the `GameInitProgress` component is registered for replication then clients can use this entity to track global loading progress.
- **`GameFwSet`**: Ordinal system set that runs in `Update`. It contains all `GameFwTickSet`s.
    - **`GameFwTickSet`**: Ordinal system sets for game app logic. All game code should go in these sets.
    - The `ClientRequestHandler` callback is invoked for all received client requests between `GameFwTickSet::Start` and `GameFwTickSet::PreLogic`. We consider request handling to be *within* the game app tick protocol, so we allow logic to be inserted before it.
- **`GameFwMode`**: Bevy state that tracks the game framework lifecycle.
    - `GameFwMode::Init` -> `GameFwMode::Game` occurs when `GameInitProgress` reaches `1.0` (i.e. all clients report they are fully initialized), or on timeout.
    - `GameFwMode::Game` -> `GameFwMode::End` occurs when the `GameEndFlag` is set.
    - `GameFwMode::End` -> `bevy::app::AppExit` occurs when `GameFwConfig::max_end_ticks()` have elapsed after entering `GameFwMode::End`. Not exiting immediately allows time to propagate the game end mode change to clients, and to allow custom app termination in game logic (i.e. by setting the max end ticks to infinite).
- **`GameEndFlag`**: Bevy resource used to signal that a game is over. Insert a `GameOverReport` to this resource with `GameEndFlag::set()` to enter `GameFwMode::End`. The report will be automatically extracted if your game is managed by a `GameInstance`.
- **`GameMessageBuffer`**: Bevy resource that you inserted to your app on app startup. All messages you want to send from the game to clients should be submitted to this buffer. Note that messages submitted to this buffer are ultimately treated as `bevy_replicon` events, which means they will synchronize with replication messages (component insertions/removals and spawns and despawns, but not component updates).

**Client Connections**

You can track and respond to client connection events with `EventReader<bevy_renet::renet::ServerEvent>`.


### Client App

**Setup**

- Implement a `ClientFactory` for making client apps. Your factory can use `prepare_girk_client_app()` to set up `bevy_girk`-related systems and resources.

**App Startup**

- Insert a `GameMessageHandler` resource to your app with your desired message-handling callback.
    - The callback can use `deserialize_game_message()?` to extract messages.
- Insert a `ClientRequestBuffer` resource to your app with your desired client request type. The request type must implement `IntoEventType` for determining the message send policy (ordered/unordered/unreliable).

**API**

The client framework exposes a small API for your client logic.

- The `GameMessageHandler` callback is invoked for all received game messages at the end of `PreUpdate` after the networking backend is done, but before any client logic has run. We consider game messages to be *authoritative* over the client app, so we handle them as soon as possible.
- **`ClientFwSet`**: Ordinal system set that runs in `Update` and contains all `ClientFwTickSet`s.
    - **`ClientFwTickSet`**: Ordinal system sets for client logic. All client code should go in these sets.
- **`ClientFwLoadingSet`**: Modal system set that runs in `Update` if the client is in mode `ClientInitializationState::InProgress`. This should contain all systems that use `iyes_progress` to track initialization progress after connecting to the game's `renet` server.
- **`ClientInitializationState`**: Bevy state that tracks whether the client is initializing or not.
    - `ClientInitializationState::InProgress` -> `ClientInitializationState::Done` is controlled by `iyes_progress`. It will only happen when all `.track_progress()` system modifiers return 100\%.
    - `ClientInitializationState::InProgress` will be set when the client disconnects. The `prepare_girk_client_app()` helper adds initialization-tracking systems to your app that prevent full initialization until the `renet` client is fully connected and the client app has received at least one `bevy_replicon` replication message.
- **`ClientFwMode`**: Bevy state that tracks the client framework lifecycle.
    - `ClientFwMode::Init` is set when initializing the client or on disconnect from the game's `renet` server (but only if the app is **not** in `ClientFwMode::End` already).
    - `ClientFwMode::Init` -> `ClientFwMode::Game` occurs when the client is in state `ClientInitializationState::Done` and the client receives a `GameFwMode::Game` message from the game framework (this message will be requested, sent, and processed automatically when `ClientInitializationState::Done` is entered).
- **`ClientRequestBuffer`**: Bevy resource that you inserted to your app on app startup. All requests you want to send from the client to the game should be submitted to this buffer.
    - *Note*: This is cleared in `PreUpdate` in order to synchronize with the networking framework. Any earlier-buffered requests will not be sent. This is relevant for e.g. GUI-based player inputs, which may be collected in an earlier schedule. It is recommended to cache player inputs in a custom buffer and marshal them into `ClientRequestBuffer` in `ClientFwSet` in `Update`. Note that this indirection makes it easier to do headless testing of the client app's core (non-graphical) logic.
- **`PingTracker`**: Bevy resource used to estimate the current game framework tick. (TODO: this is very rudimentary, it needs significant work to be production-grade)

**Reconnecting**

Ergonomic and robust client reconnects are a major feature of this library. A client app can disconnect and reconnect to the game app without needing to restart.

When a disconnect is detected, there are a number of details to keep in mind.

- The `renet` `is_disconnected()` run condition will be true for at least one full tick after a disconnect is detected.
- `ClientFwMode::Init` will be set on disconnect. There are three modalities to client initialization, which can be used to control which systems run in different scenarios (e.g. with custom system sets).
    - *Initializing*: Use `in_state(ClientFwMode::Init)`.
    - *Startup loading*: Use `in_state(ClientFwMode::Init)` plus `in_state(your custom client mode is INIT)` (assumes you track your own client mode that matches the current game mode).
    - *Reconnecting*: Use `in_state(ClientFwMode::Init)` plus `not(in_state(your custom client mode is INIT))` (assumes you track your own client mode that matches the current game mode).
- `ClientInitializationState::InProgress` will be set, which means the `ClientFwLoadingSet` will run.
- `bevy_replicon_repair` will preserve `bevy_replicon` replicated client state across a reconnect IF you use `bevy_replicon_repair` to register components for replication (e.g. with `app.replicate_repair::<MyComponent>()`).
- Client requests submitted to the `ClientRequestBuffer` will fail to send while the `renet` client is not connected (is disconnected or connecting). This ensures a clean start when the `renet` run condition `client_just_connected()` fires.
    - *Note*: There is a span of time within `ClientFwMode::Init` where all client requests will fail to send, and a span after that when ordered and unordered requests will succeed. You must use `renet` connection run conditions directly if sending requests during inititialization. Requests sent in `ClientFwMode::Game` will only fail to send if the client becomes disconnected (which may occur a number of ticks after a failed request was submitted), in which case the client app will re-initialize.
- Old server messages from the previous connection session will be discarded. New server messages will synchronize with the first replication message post-reconnect, using `bevy_replicon`'s normal message/replication synchronization mechanism.


### User Client

- setup
    - set up HostUserClient websocket client to talk to the host server
- host server interactions (mainly lobby management): see API (HostToUserMsg, HostToUserResponse, UserToHostMsg, UserToHostRequest)
- launching games
    - ClientMonitor: keeps track of the currently-running client app; receives ClientInstanceReports from the client and sends new ServerConnectTokens into the client
    - ClientStarter: convenience tool for restarting a client app that has shut down
    - launch_local_player_client(): uses
        - launches a game app binary (native: uses GameInstanceLauncherProcess)
        - launches a client app binary (native: uses ClientInstanceLauncherProcess)
    - launch_multiplayer_client()
        - launches a client app binary using data from HostToUserMsg::GameStart (native: uses GameInstanceLauncherProcess)
- client app reconnect support
    - complete restart: if a game is running and client app + user client are closed
        - on user client connect to host server, host server will automatically send HostToUserMsg::GameStart
        - initialize ClientStarter with launch_multiplayer_client()
        - use ClientStarter + ClientMonitor to launch
    - client app restart: if a game is running and client app is closed
        - expose a 'Reconnect' button to users; on press, send UserToHostRequest::GetConnectToken to host server
        - on receipt of HostToUserResponse::ConnectToken, use ClientStarter + ClientMonitor to launch
    - client app reconnect: if a game is running and the client app disconnects
        - ClientMonitor will emit a ClientInstanceReport::RequestConnectToken
        - send UserToHostRequest::GetConnectToken to host server
        - on receipt of HostToUserResponse::ConnectToken, send the token to the client app via ClientMonitor


---


## Binaries


### Backend (unified single-hub binary)

- create a host server app
- create a game hub server app
- run the apps (use `std::thread::spawn` to run one of the servers)
- note: for users to set up a HostUserClient you need the host server IP and port; how to distribute that is up to you (a future version of this architecture will include an HTTP auth server, which will provide auth tokens for creating websocket clients)


### Game App

- inprocess_game_launcher()


### Client App

- inprocess_client_launcher()


### User Client

- make {Local, Multi}PlayerLauncherConfig{Native, Wasm} and insert into app
- make HostUserClient websocket client and insert into app


### Notes

- use clap for CLI!
