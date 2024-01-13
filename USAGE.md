# Girk Usage (last updated: WIP)

This document contains a detailed description of how to use `bevy_girk` to implement a game.

For the library architecture, see `ARCHITECTURE.md`.

For a complete working example, see [bevy_girk_demo](https://github.com/UkoeHB/bevy_girk_demo).



## Overview

`bevy_girk` is designed so you can focus on writing game-specific logic. Networking and setup details are hidden by helper functions and types.

Since the backend needs to manage and launch your games, we use dependency injection to integrate your game-specific logic with `bevy_girk` servers and the game networking layer.

A project using `bevy_girk` will end up with a number of distinct apps. Each of these apps that may live in separate binaries (more on that below).

- **Game App**: A headless app with your core game logic (i.e. an 'authoritative game server').
- **Client App**: Connected to a game app, this is where users play your game.
- **Backend**:
    - **Host Server**: A headless server app that manages connected users and lobbies.
    - **Game Hub Server**: A headless server app that receives game requests from the host server and runs game apps.
- **User Client**: A GUI app that connects to, and interacts with, the host server (making/joining/starting lobbies). The app receives game start packages from the host server and launches client apps.



## Wiring

dependency injection


### Host Server

- LobbyChecker: trait object, validates lobby data, checks if new members can join, and determines whether a lobby can be launched as a game
- LobbyData: data object produced by user who creates a lobby, validated by LobbyChecker and used by GameLaunchPackSource for producing GameLaunchPacks
    - custom data


### Game Hub Server

- GameLaunchPackSource: trait object, converts GameStartRequest (containing LobbyData) into GameLaunchPack with a non-synchronous API; can be used to insert extra data into the launch pack (e.g. based on a user database query to get user loadouts)
- GameLaunchPack: data object produced by GameLaunchPackSource, used for game setup by GameFactory
    - game init data
    - ClientInitDataForGame: data object per client
        - data


### Game App

- GameFactory: trait object, uses GameLaunchPack to set up game apps, produces GameStartReports
    - GameStartReport: data object produced by the factory, used for client setup by game manager
        - auth info for creating ServerConnectToken, which are used for client setup by ClientFactory; connect tokens can be produced on-request to enable reconnects (e.g. the host server includes this in its user API)
        - GameStartInfo: data object per client, used for client setup by ClientFactory
            - start data
- ClientRequestHandler: trait object, inserted into game apps and used to handle incoming client requests
- GameMessageBuffer: bevy Resource, inserted into game apps and used to marshal game messages into the networking backend
    - GameMessage: type specified on construction, equals the type for all game network message to be sent to the client (should probably be a big enum)


### Client App

- ClientFactory: trait object, uses ServerConnectToken and GameStartInfo to set up client apps
- GameMessageHandler: trait object
- ClientRequestBuffer: bevy Resource, inserted into client apps and used to marshal client requests into the networking backend
    - ClientRequest: type specified on construction, equals the type for all client network requests to be sent to the game (should probably be a big enum)



## Implementing Apps


### Backend

- host server: make_host_server()
- game hub server: make_game_hub_server()


### Game App

- setup
    - GameFactory implementation
        - prepare_girk_game_app()
    - launching: a GameInstanceLauncher is needed to create a game hub server
        - GameInstanceLauncherProcess: launches a game app binary (note: launch_local_player_client() uses this internally)
            - use inprocess_game_launcher() inside the binary
        - GameInstanceLauncherLocal: launches a game app in a thread
- replication: use bevy_replicon_repair for registering replicated components (no components are registered by default)
- app startup
    - insert ClientRequestHandler resource with desired request-handling callback
        - use deserialize_client_request()
    - insert GameMessageBuffer resource with desired game message type (must implement IntoEventType)
- API
    - GameFwTicksElapsed
    - GameInitProgress (component on entity with `Replication` spawned by framework, if component is registered then clients can use this to track global loading progress)
    - GameFwSet: contains all GameFwTickSet, runs in Update
        - GameFwTickSet: ordinal sets for implementation logic; all game code should go in these sets
        - ClientRequestHandler callback is invoked between GameFwTickSet::Start and GameFwTickSet::PreLogic
            - request handling is *within* the game app tick protocol, so we allow inserting logic before it
    - GameFwMode
        - GameFwMode::Init -> GameFwMode::Game occurs when all clients reach ClientInitializationState::Done or on timeout
        - GameFwMode::Game -> GameFwMode::End occurs when GameEndFlag is set
    - GameMessageBuffer: send messages to clients
    - GameEndFlag: set this to enter GameFwMode::End
        - GameOverReport: data object, extracted by systems inserted by game_instance_setup()
            - game over data


### Client App

- setup
    - ClientFactory implementation
        - prepare_girk_client_app()
    - launching
        - ClientInstanceLauncherProcess: launches a client app binary (note: launch_local_player_client()/launch_multiplayer_client() use this internally)
            - use inprocess_client_launcher() inside the binary
- app startup
    - insert GameMessageHandler resource with desired message-handling callback
        - use deserialize_game_message()
    - insert ClientRequestBuffer resource with desired client request type (must implement IntoEventType)
- API
    - GameMessageHandler callback is invoked in PreUpdate after networking backend is done
        - message handling is *authoritative* over the client app tick protocol, so it runs as soon as possible
    - ClientFwSet: contains all ClientFwTickSet, runs in Update
        - ClientFwTickSet: ordinal sets for implementation logic; all user-defined client code should go here
    - ClientFwLoadingSet
        - ClientInitializationState::InProgress
    - ClientFwMode
    - ClientRequestBuffer: send messages to the game
        - note: this is cleared in PreUpdate, so any earlier buffered requests will not be sent; if UI-based player inputs are not collected in Update, it is recommended to cache them and marshal them into ClientRequestBuffer in ClientFwSet in Update
    - PingTracker: used to estimate the current game fw tick (todo: rudimentary, needs significant work to be production-grade)
- reconnecting: the renet client can disconnect and reconnect automatically (see user client for how to facilitate this)
    - renet is_disconnected() will be true for at least one full tick
    - ClientFwMode::Init will be set on disconnect
        - initializing: ClientFwMode::Init
        - startup loading: ClientFwMode::Init + client mode is init (need to track this manually)
        - reconnecting: ClientFwMode::Init + client mode is not init (need to track this manually)
    - ClientInitializationState::InProgress will be set so ClientFwLoadingSet will run
        - must reach ClientInitializationState::Done before we can leave ClientFwMode::Init (once we reach Done, the fw will request the GameFwMode; users should do the same for the game-specific mode)
    - bevy_replicon_repair will preserve bevy_replicon replicated client state; need to use bevy_replicon_repair component-registration app extension (e.g. app.replicate_repair::\<MyComponent\>())
    - client messages will fail to send while renet client is not connected
        - there is a span of time within ClientFwMode::Init where client messages will fail; must use renet connection run conditions directly if sending important messages during init
    - old server messages from the previous connection session will be discarded
        - new server messages will synchronize with the first replication message post-reconnect, using bevy_replicon's normal message/replication synchronization mechanism


### User Client

- setup
    - set up HostUserClient websocket client to talk to the host server
- host server interactions (mainly lobby management): see API (HostToUserMsg, HostToUserResponse, UserToHostMsg, UserToHostRequest)
- launching games
    - ClientMonitor: keeps track of the currently-running client app; receives ClientInstanceReports from the client and sends new ServerConnectTokens into the client
    - ClientStarter: convenience tool for restarting a client app that has shut down
    - launch_local_player_client()
        - launches a game app binary
        - launches a client app binary
    - launch_multiplayer_client()
        - launches a client app binary using data from HostToUserMsg::GameStart
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
