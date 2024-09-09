# Architecture (WIP)

This document contains an overview of `bevy_girk`'s structure and behavior.

For how to use the library, see `USAGE.md`.



## Overview

`bevy_girk` is a highly-opinionated multiplayer game architecture.

- **Game/client app framework**: This is a thin layer in your game app and client apps that interfaces with the network layer and manages initialization and reconnects. Crates: `bevy_girk_game_fw`, `bevy_girk_client_fw`.
- **Game/Client instance**: An instance encapsulates a running app. The game instance and client instance crates contain various utilities for launching and managing game and client instances. Crates: `bevy_girk_game_instance`, `bevy_girk_client_instance`.
- **Server backend**
    - **Host server**: Manages lobbies and sends new games to be launched on connected game hub servers, with basic load balancing. Allows users to reconnect to games. Crates: `bevy_girk_host_server`, `bevy_girk_backend_public`.
    - **Game hub server**: Creates and manages game instances and sends game-over reports to the host server, which are forwarded to game participants. Crates: `bevy_girk_game_hub_server`, `bevy_girk_backend_public`.
- **Wiring**: Convenience tools for setting up game and client apps. Crate: `bevy_girk_wiring`.
- **User client utils**: Convenience tools for launching and managing client apps. Crate: `bevy_girk_user_client_utils`.
- **Utils**: Generic utils library. Crate: `bevy_girk_utils`.


WORK IN PROGRESS

<!--

- architecture


- backend
    - host server
        - setup: make_host_server()
            - HostServerStartupPack: configs
                - HostServerConfig
                - LobbiesCacheConfig
                    - LobbyChecker: injected logic for evaluating lobby contents and lobby data
                - PendingLobbiesConfig
                - OngoingGamesCacheConfig
                - GameHubDisconnectBufferConfig
            - HostHubServer: bevy_simplenet server for host-hub connections
            - HostUserServer: bevy_simplenet server for host-user connections

        - behavior
            - state updates are single threaded
            - connections to users and game hubs are managed with websockets running on tokio


    - game hub server
        - setup: make_game_hub_server()
            - GameHubServerStartupPack: configs
                - GameHubServerConfig
                - PendingGamesCacheConfig
                - RunningGamesCacheConfig
            - Receiver\<GameHubCommand\>: sends commands to the game hub server (e.g. shut down, modify capacity)
            - HostHubClient: bevy_simplenet client for host-hub  connections
            - GameLaunchPackSource: converts lobbies into game launch packs
            - GameInstanceLauncher: launches game instances from launch packs

        - behavior
            - state updates are single threaded
            - connection to the host server is managed with websockets running on tokio


- backend public
    - LobbyChecker: trait
    - GameLaunchPackSource: boxed trait
    - host-user websocket channel types
    - lobby types


- framework
    - game framework
        - setup
            - prepare_girk_game_app()
            - ClientRequestHandler: boxed trait, ...

        - exposed contents
            - GameFwSet
            - GameFwState
            - GameMessageBuffer

    - client framework
        - setup
            - prepare_girk_client_app()
            - GameMessageHandler: boxed trait, ...

        - exposed contents
            - ClientFwSet
            - ClientFwLoadingSet
            - ClientFwState
            - ClientRequestSender


- management
    - game instance
        - GameInstance: interface for monitoring a running game instance; can send in commands

        - GameFactory: boxed trait, portable tool for creating game apps

        - GameInstanceLauncher: boxed trait, produces GameInstances
            - GameInstanceLauncherLocal
                - setup
                - behavior
            - GameInstanceLauncherProcess
                - setup
                - behavior

        - game_instance_setup(): sets up a game app as a game instance app
            - GameFactory
            - GameLaunchPack
            - IoSender\<GameInstanceReport\>
            - IoReceiver\<GameInstanceCommand\>

            - behavior
                - extracts GameOverReport from app and forwards to the instance owner as a GameInstanceReport

    - client instance
        - ClientInstance: interface for monitoring a running client instance

        - ClientFactory: boxed trait, portable tool for creating client apps

        - ClientInstanceLauncher: boxed trait, produces ClientInstances
            - ClientInstanceLauncherProcess
                - setup
                - behavior

        - client_instance_setup()
            - ClientFactory
            - ServerConnectToken
            - GameStartInfo
            - ClientInstanceConfig
            - IoSender\<ClientInstanceReport\>
            - IoReceiver\<ClientInstanceCommand\>

            - behavior
                - connects reconnect logic of app with the app owner
                    - requests ServerConnectTokens from the instance owner on disconnect via ClientInstanceReport
                    - receives ServerConnectToken via ClientInstanceCommand, inserts to app as RenetClientConnectPack resource, then the systems from prepare_girk_client_app() will use it to set up a new renet client

    - user client utils
        - launchers: practical methods for launching clients (automatic native/WASM support {WASM is WIP})
            - launch_local_player_client()
            - launch_multiplayer_client()
        - UserClientUtilsPlugin
            - ClientMonitor
            - ClientStarter


- binaries
    - backend (unified single-hub)


    - game instance


    - client instance



- utils
    - child process utils
    - network setup
    - network utils
    - rand64

-->
