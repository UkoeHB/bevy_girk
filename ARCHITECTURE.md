# Girk Architecture (last updated: WIP)

This document contains a detailed overview of the library's structure and how it can be used to implement a game.



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
            - GameFwMode
            - GameMessageBuffer

    - client framework
        - setup
            - prepare_girk_client_app()
            - GameMessageHandler: boxed trait, ...

        - exposed contents
            - ClientFwSet
            - ClientFwLoadingSet
            - ClientFwMode
            - ClientRequestBuffer


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
    - game instance


    - client instance



- utils
    - child process utils
    - network setup
    - network utils
    - rand64

