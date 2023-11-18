Game framework: `GameFWPlugin`

PRECONDITION: the following must be initialized by the user
- `Res<GameFWConfig>`
- `Res<GameFWInitializer>`
- `Res<Sender<GamePacket>>`
- `Res<Receiver<ClientPacket>>`

PRECONDITION: the following must be initialized by the game core that uses this game framework
- `Res<ClientMessageHandler>`

INTERFACE: for game framework manager
- the client message channels should be connected in some way to the corresponding channels in the client framework
- the system set `GameFWSet` should be configured to run at the appropriate time in the bevy scheduling context
- the state `GameFWMode` may be read to track progress through the game `(Init -> Game -> End)`

INTERFACE: for game core
- game logic should be defined within system set `GameFWSet`
- `Res<GameEndFlag>` must be set with a game over report to terminate the game
- `Res<GameMessageBuffer>` allows game messages to be sent to clients
- `Res<GameFWConfig>` provides miscellaneous information about the client
- `Res<ClientEntityMap>` can be used to translate between client id and client entity
- `State<GameFWMode>` tracks what mode the game framework is in
- client entities are loaded into the ECS; they can be manipulated/adjusted as needed
- file `utils.rs` provides various useful public functions


TODO:
- entity and event access should be controlled via bevy_replicon Rooms
