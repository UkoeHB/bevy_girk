# Usage

This document contains a detailed description of how to use `bevy_girk` to implement a game.

For the library architecture, see `ARCHITECTURE.md`.

To navigate the types and functions in this document it is recommended to open it in a text editor and use 'go to definition'.



## Overview

`bevy_girk` is designed so you can focus on writing game-specific logic. Networking and setup details are hidden by helper functions and types.

Since the backend needs to manage and launch your games, we use dependency injection to integrate your game-specific logic with `bevy_girk` servers and the game networking layer.

A project using `bevy_girk` will end up with a number of distinct apps. Each of those apps may or may not live in separate binaries (more on that below).

- **Game App**: A headless app with your core game logic (i.e. a 'single-threaded authoritative game server'). Game messaging and state replication are done with `bevy_replicon`, `bevy_replicon_repair`, and `bevy_replicon_attributes` using a `renet` server/client setup.
- **Client App**: A GUI app connected to a game app, this is where users play your game.
- **Backend**:
    - **Host Server**: A headless server app that manages connected users and lobbies. Networking uses `bevy_simplenet`.
    - **Game Hub Server**: A headless server app that receives game requests from the host server and runs game apps. Networking uses `bevy_simplenet`.
- **User Client**: A GUI app that connects to, and interacts with, the host server (making/joining/starting lobbies). The app receives game start packages from the host server and launches client apps. It is also helps coordinate client app reconnects for ongoing games.



## Wiring

Dependency injection ties your game-specific logic to the `bevy_girk` servers and networking layer. A `bevy_girk` project needs to implement a number of trait objects and define how to serialize/deserialize various data objects.


### Host Server

- **`LobbyChecker`** (trait object): Validates `LobbyData`, checks if new members can join a lobby, and determines whether a lobby can be launched as a game.
- **`LobbyData`** (data object): Produced by a user who creates a lobby, validated by `LobbyChecker` and consumed by `GameLaunchPackSource` for producing `GameLaunchPack`s. When the data changes during lobby setup (e.g. as members join/leave), the host server sends this data to lobby members so it can be used to display up-to-date lobby information.
    - The custom data field in this type should deserialize to your custom lobby data.


### Game Hub Server

- **`GameLaunchPack`** (data object): Produced by a `GameLaunchPackSource`, used for game app setup by `GameFactory`. 
    - The game launch data field in this type should deserialize to game-specific initialization details for a game.
- **`GameLaunchPackSource`** (trait object): Converts `GameStartRequest`s (containing `LobbyData`s) into `GameLaunchPack`s with an asynchronous API. This allows you to insert extra data into launch packs beyond just the lobby data (e.g. based on a user database query to get user loadouts).
- **`GameInstanceLauncher`** (trait object): Launches a game app, returns a `GameInstance` for managing the game. Two default implementations are provided:
    - `GameInstanceLauncherProcess`: Launches a game app binary in a child process.
        - Use `inprocess_game_launcher()` inside the binary. This helper takes a `GameFactory` to create your game app.
    - `GameInstanceLauncherLocal`: Launches a game app in a `std::thread` using a `GameFactory`.


### Game App

- **`GameFactory`** (trait object): Consumes `GameLaunchPack`s to set up game apps, and produces `GameStartReport`s.
    - Note that client ids you specify in the start report will be used in-game to interact with clients, so they should be mapped to player data correctly.
- **`GameStartReport`** (data object): Produced by a `GameFactory`, used to orchestrate client setup by the game manager (for non-singleplayer games this is the host server).
    - Includes auth info for creating `ServerConnectToken`s, which are used by client apps to connect to the game app's `renet` server. Start reports include auth info instead of connect tokens so that tokens can be produced on-demand by the host server to support reconnects.
    - **`GameStartInfo`** (data object): Per-client custom data, used for client setup by `ClientFactory`. The start data field in this type should deserialize to game-specific initialization details for a client app.
- **`GameOverReport`** (data object): A report that is submitted by custom game logic to the game app's `GameEndFlag` resource. It will be extracted by systems inserted by `game_instance_setup()`. For multiplayer games, the report will ultimately be forward to the user client in a `HostToUserMsg::GameOver` message. The game over data field in this type should deserialize to a game-specific game over report.
- **`ClientRequestHandler`** (trait object): Bevy resource inserted into game apps and used to handle incoming client requests.
- **`GameMessageType`**: Bevy resource inserted into game apps and used to validate the type of game messages submitted to `GameMessageSender`.
    - *Note*: We use type id consistency checks in the `GameMessageSender` instead of making it generic so the game framework doesn't need to have generic functions and types everywhere.


### Client App

- **`ClientFactory`** (trait object): Consumes `ServerConnectToken`s and `GameStartInfo`s to set up client apps.
- **`GameMessageHandler`** (trait object): Bevy resource inserted into client apps and used to handle incoming game messages.
- **`ClientRequestType`**: Bevy resource inserted into client apps and used to validate the type of game messages submitted to `ClientRequestSender`.
    - *Note*: We use type id consistency checks in the `ClientRequestSender` instead of making it generic so the client framework doesn't need to have generic functions and types everywhere.



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

The game app is a single-threaded authoritative server where game logic is executed. Clients communicate with the game app via a `renet` server/client relationship, and game state is replicated to clients with `bevy_replicon`/`bevy_replicon_repair`. Networking details are mostly hidden by `bevy_girk` APIs.

**Setup**

- Implement a `GameFactory` for making game apps. Your factory should use `prepare_girk_game_app()` to set up `bevy_girk`-related systems and resources (if you don't use that helper than none of the information below will be accurate).

**Replication**

- Use [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair) to register replicated components. Only the `GameInitProgress` component is registered by default. Note that components must be registered in the same order in the game and client. This means you should always call `prepare_girk_game_app()` and `prepare_girk_client_app()` before your core app logic in your game/client factories (otherwise `GameInitProgress` may be registered out of order).
- Include a [bevy_replicon](https://github.com/lifescapegame/bevy_replicon) `Replication` component in entities that should be replicated.

**Visibility Control**

Visibility of entities and game messages is controlled by [bevy_replicon_attributes](https://github.com/UkoeHB/bevy_replicon_attributes).
- Game entities *will not replicate* if you do not give them a `VisibilityCondition` component. Use `vis!(Global)` if you want all clients to see an entity. See the `bevy_replicon_attributes` crate for more details.
- Use `bevy_replicon_attributes::ClientAttributes` to add/remove client attributes (to control which visibility conditions each client satisfies).
- Use `GameMessageSender` to send game messages filtered by visibility conditions.

**App Startup**

- Insert a `ClientRequestHandler` resource to the app with your desired request-handling callback.
    - The callback should use `deserialize_client_request()?` to extract requests. This function is exposed in case you want to read `ClientFwRequest`s sent by the client framework.
- Insert a `GameMessageType` resource to the app with your desired game message type. The message type must implement `IntoEventType` for determining the message send policy (ordered/unordered/unreliable).

**API**

The game framework exposes a small API to support your game logic.

- **`GameFwConfig`**: Bevy resource that can be used to get the game's tick rate.
- **`GameFwClients`**: Bevy resource that can be used to iterate the client list within the game.
- **`GameFwTick`**: Bevy resource that records the current game tick. See the `GameFwTickPlugin` code docs for more details.
- **`GameInitProgress`**: Bevy component on a replicated entity spawned by the framework at startup with global visibility. It tracks the total initialization progress of all clients while the game is initializing. The client framework will automatically reset the replicated progress when the client disconnects.
- **`GameFwSet`**: Ordinal system sets for game app logic. All game code should go in these sets.
    - Client requests are handled by `ClientRequestHandler` in a private set that runs between `GameFwSet::Start` and `GameFwSet::PreLogic`. We consider request handling to be *within* the game app tick protocol, so we allow logic to be inserted before it.
- **`GameFwMode`**: Bevy state that tracks the game framework lifecycle.
    - `GameFwMode::Init` -> `GameFwMode::Game` occurs when `GameInitProgress` reaches `1.0` (i.e. all clients report they are fully initialized), or on timeout.
    - `GameFwMode::Game` -> `GameFwMode::End` occurs when the `GameEndFlag` is set.
    - `GameFwMode::End` -> `bevy::app::AppExit` occurs when `GameFwConfig::max_end_ticks()` have elapsed after entering `GameFwMode::End`. Not exiting immediately allows time to propagate the game end mode change to clients, and to allow custom app termination in game logic (i.e. by setting the max end ticks to infinite).
- **`GameEndFlag`**: Bevy resource used to signal that a game is over. Insert a `GameOverReport` to this resource with `GameEndFlag::set()` to enter `GameFwMode::End`. The report will be automatically extracted if your game is managed by a `GameInstance`.
- **`ClientReadiness`**: Bevy resource that tracks the readiness of clients (i.e. how close they are to being ready to play). Note that client readiness logic is automatically handled by `bevy_girk` systems, so you should not need to use `ClientReadiness::set()`. Client readiness is reset when a client disconnects.
- **`GameMessageSender`**: Bevy system parameter that allows you to send game messages to clients. Uses `GameMessageType` to validate game message types when `debug_assertions` are enabled.
    - *Note*: Messages submitted to this buffer are ultimately treated as `bevy_replicon` events, which means they will synchronize with replication messages (component insertions/removals and spawns and despawns, but not component updates).

**Client Connections**

You can track and respond to client connection events with `EventReader<bevy_renet::renet::ServerEvent>`. It is recommended to handle these events promptly every tick if your client-specific state needs to synchronize with connection events.


### Client App

The client app is a GUI where you play a game. Clients communicate with the game app through a `renet` server/client relationship. Game state is replicated into the client app with `bevy_replicon`/`bevy_replicon_repair`. Networking details are mostly hidden by `bevy_girk` APIs.

**Setup**

- Implement a `ClientFactory` for making client apps. Your factory should use `prepare_girk_client_app()` to set up `bevy_girk`-related systems and resources (if you don't use that helper than none of the information below will be accurate).

**App Startup**

- Insert a `GameMessageHandler` resource to your app with your desired message-handling callback.
    - The callback should use `deserialize_game_message()?` to extract messages. This function is exposed in case you want to read `GameFwMsg`s sent by the game framework.
- Insert a `ClientRequestType` resource to your app with your desired client request type. The request type must implement `IntoEventType` for determining the message send policy (ordered/unordered/unreliable).

**API**

The client framework exposes a small API to support your client logic.

- Game messages are handled by `GameMessageHandler` at the end of `PreUpdate` after the networking backend is done, but before any client logic has run. We consider game messages to be *authoritative* over the client app, so we handle them as soon as possible.
- **`ClientFwSet`**: Ordinal system sets that run in `Update` and should contain all client logic.
- **`ClientFwLoadingSet`**: Modal system set in `Update` that runs if the client is in mode `ClientInitializationState::InProgress`. This should contain all systems that use `iyes_progress` to track initialization progress.
- **`ClientInitializationState`**: Bevy state that tracks whether the client is initializing or not.
    - `ClientInitializationState::InProgress` -> `ClientInitializationState::Done` is controlled by `iyes_progress`. It will only happen when all `.track_progress()` systems return 100\%.
    - `ClientInitializationState::InProgress` will be set when the client disconnects. The `prepare_girk_client_app()` helper adds initialization-tracking systems to your app that prevent full initialization until after the first tick of `ClientFwMode::Init`. This means your app will be in `ClientInitializationState::InProgress` for the entirety of `ClientFwMode::Connecting` and `ClientFwMode::Syncing`, plus at least one tick of `ClientFwMode::Init`.
- **`ClientFwMode`**: Bevy state that tracks the client framework lifecycle.
    - `ClientFwMode::Connecting` is set when initializing the client or on disconnect from the game's `renet` server (but only if the app is **not** in `ClientFwMode::End` already). This mode always runs for at least one tick during an initialization cycle, and in its first tick the `renet` `client_just_disconnected()` run condition will be true.
    - `ClientFwMode::Syncing` is set when the client has connected to the `renet` server and is waiting for `bevy_replicon`'s first replication message. No server messages will be consumed in this mode `bevy_replicon` forces server messages to synchronize with replication state; received messages will be buffered until `ClientFwMode::Init`. This mode always runs for at least one tick during an initialization cycle, and in its first tick `renet`'s `client_just_connected()` run condition will be true.
    - `ClientFwMode::Init` is set when the client has synchronized with the server's replication state. The client may not be fully synchronized, however, if there are unreceived server messages required to fully initialize. This mode always runs for at least one tick during an initialization cycle.
    - `ClientFwMode::Init` -> `ClientFwMode::Game` occurs when the client is in `ClientInitializationState::Done` and the client receives a `GameFwMode::Game` message from the game framework (the `GameFwMode` will be requested, sent by the server, and processed automatically by the client when `ClientInitializationState::Done` is entered).
- **`ClientRequestSender`**: Bevy system parameter used to send client requests to the game. Uses `ClientRequestType` to validate client request types when `debug_assertions` are enabled.
    - *Note*: This uses `bevy_replicon` events under the hood, which means client requests properly synchronize with client connection cycles. Client requests will be dropped between a disconnect and entering `ClientFwMode::Syncing`.
- **`PingTracker`**: Bevy resource used to estimate the current game framework tick. (TODO: this is very rudimentary, it needs significant work to be production-grade)

**Reconnecting**

Robust, ergonomic client reconnects are a major feature of this library. A disconnected client app will automatically try to reconnect to the game app without needing to restart.

When a disconnect is detected, there are a number of details to keep in mind.

- There are two dimensions to client initialization, which can be used to control which systems run in different scenarios (e.g. with custom system sets).
    - *Framework initialization cycle*: The framework moves through these states every time your client app connects/reconnects: `ClientFwMode::Connecting` -> `ClientFwMode::Syncing` -> `ClientFwMode::Init`.
    - *Client initialization types*: Usually multiplayer games are divided into an initial loading phase and then a game phase. The game phase can contain multiple reconnect cycles. You can track your game's state to distinguish between startup loading and reconnecting (which may require different loading screens and systems).
- `ClientInitializationState::InProgress` will be set on disconnect, which means the `ClientFwLoadingSet` will run.
- `bevy_replicon` replicated client state will be preserved across a reconnect if you use `bevy_replicon_repair` to register components for replication (e.g. with `app.replicate_repair::<GameInitProgress>()`). If you don't use `bevy_replicon_repair` then you need to manually repair your client state (e.g. by despawning and cleaning up all old replicated entities when entering `ClientFwMode::Syncing`, although note that this may cause a visual glitch for the duration of this mode).
- Client requests submitted to the `ClientRequestSender` will fail to send while in `ClientFwMode::Connecting`. This ensures a clean start when you enter `ClientFwMode::Syncing`. Note that a disconnect event always occurs at an ambiguous point in time. In practice your client messages will fail to send from that ambiguous disconnect point until you enter `ClientFwMode::Syncing` for the reconnect cycle, and then will succeed until another disconnect occurs (which will trigger another reconnect cycle to repair the client).
- Old server messages from the previous connection session will be discarded. New server messages will synchronize with the first replication message post-reconnect, using `bevy_replicon`'s normal message/replication synchronization mechanism. This means you won't process any server messages until you enter `ClientFwMode::Init` (messages received in `ClientFwMode::Syncing` are buffered).


### User Client

The user client app is a GUI that communicates with the host server and launches client apps. It is possible to build an integrated user client/game client app, however the setup helpers currently provided by `bevy_girk` assume they are separate apps in separate processes.

**Setup**

- Set up a `HostUserClient` `bevy_simplenet` client to talk to the host server.
    - Most host server interactions are related to lobby management (making/joining/leaving/launching/searching lobbies). For the entire API see `HostToUserMsg`, `HostToUserResponse`, `UserToHostMsg`, `UserToHostRequest`.
- Add the `ClientMonitorPlugin` to your app.
    - **`ClientMonitor`**: Bevy resource inserted by `ClientMonitorPlugin` that keeps track of the currently-running client app if there is one. It receives `ClientInstanceReport`s from the client and can send new `ServerConnectToken`s into the client.
- Add the `ClientStarterPlugin` to your app.
    - **`ClientStarter`**: Bevy resource inserted by `ClientStarterPlugin` that makes it easy to restart a client app that has shut down.

**Launching Local Single-Player Game Clients**

1. Make a `GameLaunchPack` for the game.
1. Use `launch_local_player_client()` to launch the game (no network connection required).
    - Internally this launches your game app binary. On native it uses `GameInstanceLauncherProcess`. TODO: WASM
    - Internally this launches your client app binary. On native it uses `ClientInstanceLauncherProcess`. TODO: WASM

**Launching Hosted Multiplayer Game Clients**

Hosted gamed should use the `ClientStarter` to support reconnects that need to re-use `GameStartInfo`.

1. Receive a `HostToUserMsg::GameStart` from the host server. This includes `ServerConnectToken` and `GameStartInfo`.
1. Set the client starter: `ClientStarter::set()`.
    - Use `launch_multiplayer_client()` in the starter callback to launch the game. Move the `GameStartInfo` into the callback and clone it there (the callback is `FnMut`).
        - Internally this launches your client app binary. On native it uses `ClientInstanceLauncherProcess`. TODO: WASM
1. Launch the client with `ClientStarter::start()`.

**Hosted Multiplayer Reconnect Support**

The user client helps reconnect users to hosted games. There are three primary kinds of reconnects.

- **Complete restart**: A hosted game is running and the client app and user client are both closed.
    1. When the user client reopens and connects to the host server, the host server will automatically send a `HostToUserMsg::GameStart` containing `ServerConnectToken` and `GameStartInfo`.
    1. Call `ClientStarter::set()` with `launch_multiplayer_client()` in the callback. Move the `GameStartInfo` into the callback and clone it there (the callback is `FnMut`).
    1. Launch the client with `ClientStarter::start()`.
- **Client app restart**: A hosted game is running and the client app is closed but the user client is open and connected.
    1. Expose a 'Reconnect' button to users, which will display when `ClientStarter` has been set (i.e. it was set by a game start but not cleared by a game over, implying a hosted game is still running). On press, send `UserToHostRequest::GetConnectToken` to the host server.
    1. On receipt of `HostToUserResponse::ConnectToken`, use the already-set `ClientStarter` to launch the client app with `ClientStarter::start()`.
- **Client app reconnect**: A hosted game is running and the client app disconnected but remains open (and the user client is open and connected).
    1. The `ClientMonitor` will emit a `ClientInstanceReport::RequestConnectToken`.
    1. Send `UserToHostRequest::GetConnectToken` to the host server.
    1. On receipt of `HostToUserResponse::ConnectToken`, send the token to the client app via `ClientMonitor::send_token()`.

There is also the scenario where the user client disconnects while remaining open (with the client app either open or closed). When the user client reconnects it will receive a `HostToUserMsg::GameStart` from the host server, which can be used to open a new client app immediately if there isn't one already, or you can send the connect token into the client monitor. If the host server sends you information for a new game, then the existing client needs to be force-closed and reopened.


## Binaries

Your project will need several binary crates to get started.


### Backend (unified single-hub binary)

For testing it is recommended to build one unified binary for the backend.

1. Create a host server app.
1. Create a game hub server app.
1. Run the apps. Use `std::thread::spawn` to run one of the servers in another thread.

*Note*: For user clients to set up a `HostUserClient` you need the host server IP and port. How to distribute that information is up to you. A future version of `bevy_girk` will include an HTTP auth server, which will provide auth tokens to clients for creating `HostUserClient`s. In that case you'd only need to distribute the auth server's address (which is easier to bake into client binaries).


### Game App

Your game app should have its own binary. This binary can be launched by the game hub server for hosted games and by the user client for local single-player games.

1. Use `inprocess_game_launcher()` in your `fn main()`.


### Client App

Your client app should have its own binary. This binary can be launched by the user client for users to play your game.

1. Use `inprocess_client_launcher()` in your `fn main()`.


### User Client

Your user client should also have a binary.

1. You need to set up launcher configs in order to call `launch_local_player_client()` and `launch_multiplayer_client()` within your app. These should be made in your `fn main()` and inserted into the app. They are (for native and WASM environments): `LocalPlayerLauncherConfigNative`, `MultiPlayerLauncherConfigNative`, `LocalPlayerLauncherConfigWasm`, `MultiPlayerLauncherConfigWasm`.
1. Make a `HostUserClient` `bevy_simplenet` client and insert it into the app.


### Notes

- Use [`clap`](https://docs.rs/clap/4.4.11/clap/index.html) for CLI!
