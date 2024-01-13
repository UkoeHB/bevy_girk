# Bevy game instance framework

A framework for multiplayer games where the player list is known before starting a game and games last a few hours at most.

For a complete working example, see [bevy_girk_demo](https://github.com/UkoeHB/bevy_girk_demo).

For details about this project, see `USAGE.md` and `ARCHITECTURE.md`.


### Dependencies

In-game networking is implemented with [bevy_replicon](https://github.com/lifescapegame/bevy_replicon), [bevy_renet](https://github.com/lucaspoffo/renet), and [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair). Game clients can be set up to work on both native and WASM targets (pending a `bevy_renet` WASM transport).

Backend networking is implemented with [bevy_simplenet](https://github.com/UkoeHB/bevy_simplenet). User clients will work on both native and WASM targets.



### Disclaimer

This project is only the starting point for a multiplayer game architecture. Future work involves greatly expanding the server backend to include user authentication, a user profile database, matchmaking, match history, rankings, auto-updates for native client binaries, etc.

There may be limits to how far this framework can go while remaining generic, so this project may need to stabilize at a partial/minimal solution.
