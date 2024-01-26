Game framework: `GameFwPlugin`

PRECONDITION: the following must be initialized by the user
- `Res<GameFwClients>`
- `Res<GameFwConfig>`

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
- `ServerManager` allows game messages to be sent to clients
- `Res<GameFwClients>` provides the game's client list
- `Res<GameFwConfig>` provides setup information about the game
- `State<GameFwMode>` tracks what mode the game framework is in

See `USAGE.md` for more information.
