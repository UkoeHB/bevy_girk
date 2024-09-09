Game framework: `GameFwPlugin`

PRECONDITION: the following must be initialized by the user
- `Res<GameFwClients>`
- `Res<GameFwConfig>`

PRECONDITION: the following must be initialized by the game core that uses this game framework
- `Res<ClientRequestHandler>`
- `Res<GameMessageType>`

INTERFACE: for game framework manager
- The client message channels should be connected in some way to the corresponding channels in the client framework.
- The `GameFwSet` system sets should be configured to run at the appropriate time in the bevy scheduling context.
- The state `GameFwState` may be read to track progress through the game `(Init -> Game -> End)`.

INTERFACE: for game core
- Game logic should be defined within the `GameFwSet` system sets.
- `Res<GameEndFlag>` must be set with a game over report to terminate the game
- `GameMessageSender` allows game messages to be sent to clients
- `Res<GameFwClients>` provides the game's client list
- `Res<GameFwConfig>` provides setup information about the game
- `State<GameFwState>` tracks what state the game framework is in

See `USAGE.md` for more information.
