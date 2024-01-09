Game framework: `GameFwPlugin`

PRECONDITION: the following must be initialized by the user
- `Res<GameFwConfig>`
- `Res<GameFwInitializer>`
- `Res<Sender<GamePacket>>`
- `Res<Receiver<ClientPacket>>`

PRECONDITION: the following must be initialized by the game core that uses this game framework
- `Res<ClientRequestHandler>`
- `Res<GameMessageBuffer>`

INTERFACE: for game framework manager
- the client message channels should be connected in some way to the corresponding channels in the client framework
- the system set `GameFwSet` should be configured to run at the appropriate time in the bevy scheduling context
- the state `GameFwMode` may be read to track progress through the game `(Init -> Game -> End)`

INTERFACE: for game core
- game logic should be defined within system set `GameFwSet`
- `Res<GameEndFlag>` must be set with a game over report to terminate the game
- `Res<GameMessageBuffer>` allows game messages to be sent to clients
- `Res<GameFwConfig>` provides miscellaneous information about the client
- `Res<ClientEntityMap>` can be used to translate between client id and client entity
- `State<GameFwMode>` tracks what mode the game framework is in
- client entities are loaded into the ECS; they can be manipulated/adjusted as needed
- file `utils.rs` provides various useful public functions


TODO:
- entity and event access should be controlled via bevy_replicon Rooms
