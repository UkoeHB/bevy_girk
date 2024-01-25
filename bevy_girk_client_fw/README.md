Client framework: `ClientFwPlugin`

PRECONDITION: plugin dependencies
- `bevy::time::TimePlugin`

PRECONDITION: the following must be initialized by the client framework manager
- `Res<ClientFwConfig>`
- `Res<Receiver<ClientFwCommand>>`

PRECONDITION: the following must be initialized by the client core that uses this client framework
- `Res<GameMessageHandler>`
- `Res<ClientRequestType>`

INTERFACE: for client framework manager
- the `ClientFwCommand` channel may be used to control the client framework
- the client message channels should be connected in some way to the corresponding channels in the game framework
- the system set `ClientFwSet` should be configured to run at the appropriate time in the bevy scheduling context

INTERFACE: for users of the framework (framework owners and client dependents)
- game logic should be defined within system set `ClientFwSet`
- `Res<PingTracker>` provides game-synchronization information
- `ClientRequestSender` collects requests to be sent to the game
- `Res<ClientFwConfig>` provides miscellaneous information about the client
- `State<ClientFwMode>` tracks what mode the client is in
- `Res<InitializationProgressCache>` may be used to track the progress of initialization
- `iyes_progress`: add `.track_progress().in_set(ClientFwLoadingSet)` to a system to track it during initialization
