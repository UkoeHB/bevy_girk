Client framework: `ClientFWPlugin`

PRECONDITION: plugin dependencies
- `bevy::time::TimePlugin`

PRECONDITION: the following must be initialized by the client framework manager
- `Res<ClientFWConfig>`
- `Res<MessageReceiver<GamePacket>>`
- `Res<MessageSender<ClientPacket>>`
- `Res<MessageReceiver<ClientFWCommand>>`

PRECONDITION: the following must be initialized by the client core that uses this client framework
- `Res<GameMessageHandler>`

INTERFACE: for client framework manager
- the `ClientFWCommand` channel may be used to control the client framework
- the client message channels should be connected in some way to the corresponding channels in the game framework
- the system set `ClientFWSet` should be configured to run at the appropriate time in the bevy scheduling context

INTERFACE: for users of the framework (framework owners and client dependents)
- game logic should be defined within system set `ClientFWSet`
- `Res<PingTracker>` provides game-synchronization information
- `Res<ClientMessageBuffer>` collects messages to be sent to the game
- `Res<ClientFWConfig>` provides miscellaneous information about the client
- `State<ClientFWMode>` tracks what mode the client is in
- `Res<InitializationProgressCache>` may be used to track the progress of initialization
- `[iyes_progress]`: add `.track_progress().in_set(ClientFWLoadingSet)` to a system to track it during initialization
