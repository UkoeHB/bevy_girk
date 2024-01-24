# Bevy game instance framework

A framework/architecture for match-based multiplayer games.

For a complete working example, see [bevy_girk_demo](https://github.com/UkoeHB/bevy_girk_demo).

For details about this project, see `USAGE.md` and `ARCHITECTURE.md`.


### Networking

- In-game networking is implemented with [bevy_replicon](https://github.com/lifescapegame/bevy_replicon), [bevy_renet](https://github.com/lucaspoffo/renet), and [bevy_replicon_repair](https://github.com/UkoeHB/bevy_replicon_repair).
- Backend networking is implemented with [bevy_simplenet](https://github.com/UkoeHB/bevy_simplenet).

Native/WASM compatibility is a big goal of this project, however WASM is not yet supported pending a `bevy_renet` WASM transport.


### Disclaimer

This project is only the starting point for a multiplayer game architecture. Future work involves greatly expanding the server backend to include user authentication, a user profile database, matchmaking, match history, rankings, auto-updates for native client binaries, etc.

There may be limits to how far `bevy_girk` can go while remaining generic, so this project may need to stabilize at a partial/minimal solution.


### Major TODOs

- Redesign framework system sets to incorporate `FixedUpdate`.
- Add an authentication server so `bevy_girk` games can plausibly be deployed in production (or at least alpha nets).
- Refactor everything to use generics to eliminate multi-layered serialization/deserialization. Alternatively, write custom serializers/deserializers for data objects that reuse existing allocations (via `Bytes`).
- Improve host server design to increase capacity. The logic loop is currently single-threaded, but maybe some work can be offloaded to other threads (e.g. lobby searches). It's important to maintain synchronization guarantees.
