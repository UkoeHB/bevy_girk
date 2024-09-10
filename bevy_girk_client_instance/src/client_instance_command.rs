//local shortcuts
use crate::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::ecs::world::Command;
use bevy::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A command that may be sent in a client to set up or shut down a game.
///
/// These commands cause [`ClientInstanceState`] state transitions, and send [`ClientInstanceReport`] events.
#[derive(Debug, Clone)]
pub enum ClientInstanceCommand
{
    /// Setup game.
    Start(ServerConnectToken, GameStartInfo),
    /// Close the client instance in order to get a new connect token.
    RequestConnectToken,
    /// End the client instance.
    End,
    /// Abort the client instance.
    Abort,
}

impl Command for ClientInstanceCommand
{
    fn apply(self, w: &mut World)
    {
        match self {
            Self::Start(token, start_info) => {
                tracing::debug!("starting game {}", start_info.game_id);
                w.resource_scope(|w: &mut World, mut factory: Mut<ClientFactory>| {
                    factory.setup_game(w, token, start_info);
                });
                set_and_apply_state(w, ClientInstanceState::Game);
            }
            Self::RequestConnectToken => {
                if *w.resource::<State<ClientInstanceState>>().get() != ClientInstanceState::Game {
                    tracing::warn!("ignoring command to get a new connect token, client is not in a game");
                    return;
                }
                let game_id = w.resource::<ClientFwConfig>().game_id();
                tracing::warn!("closing game {game_id} to request connect token");
                w.resource_mut::<Events<ClientInstanceReport>>().send(ClientInstanceReport::RequestConnectToken);
                set_and_apply_state(w, ClientInstanceState::Client);
            }
            Self::End => {
                if *w.resource::<State<ClientInstanceState>>().get() != ClientInstanceState::Game {
                    tracing::warn!("ignoring command to end game, client is not in a game");
                    return;
                }
                let game_id = w.resource::<ClientFwConfig>().game_id();
                tracing::debug!("ending game {game_id}");
                w.resource_mut::<Events<ClientInstanceReport>>().send(ClientInstanceReport::Ended(game_id));
                set_and_apply_state(w, ClientInstanceState::Client);
            }
            Self::Abort => {
                if *w.resource::<State<ClientInstanceState>>().get() != ClientInstanceState::Game {
                    tracing::warn!("ignoring command to abort game, client is not in a game");
                    return;
                }
                let game_id = w.resource::<ClientFwConfig>().game_id();
                tracing::warn!("aborting game {game_id}");
                w.resource_mut::<Events<ClientInstanceReport>>().send(ClientInstanceReport::Aborted(game_id));
                set_and_apply_state(w, ClientInstanceState::Client);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
