//local shortcuts
use crate::{ClientFactory, ClientInstanceReport, LocalGameManager};
use bevy_girk_client_fw::{ClientFwConfig, ClientAppState};
use bevy_girk_game_instance::{GameLaunchPack, GameStartInfo};
use bevy_girk_utils::set_and_apply_state;

//third-party shortcuts
use bevy::prelude::*;
use renet2_setup::ServerConnectToken;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// A command that may be sent in a client to set up or shut down a game.
///
/// These commands cause [`ClientAppState`] state transitions, and send [`ClientInstanceReport`] events.
#[derive(Debug, Clone)]
pub enum ClientInstanceCommand
{
    /// Setup game.
    ///
    /// The client will move to [`ClientAppState::Connecting`] after one tick.
    Start(ServerConnectToken, GameStartInfo),
    /// Setup local-player game.
    ///
    /// Once the game app has emitted a [`GameStartReport`], the client will move to
    /// [`ClientAppState::Connecting`].
    StartLocal(GameLaunchPack),
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
                set_and_apply_state(w, ClientAppState::Game);
            }
            Self::StartLocal(launch_pack) => {
                tracing::debug!("starting local game {}", launch_pack.game_id);

                // Launch game app.
                // - The client will be set up automatically once a GameStartReport is detected.
                // - We do not enter ClientAppState::Game until the game start report is available.
                w.resource_mut::<LocalGameManager>().launch(launch_pack);
            }
            Self::RequestConnectToken => {
                if *w.resource::<State<ClientAppState>>().get() != ClientAppState::Game {
                    tracing::warn!("ignoring command to get a new connect token, client is not in a game");
                    return;
                }
                let game_id = w.resource::<ClientFwConfig>().game_id();
                tracing::warn!("closing game {game_id} to request connect token");
                w.resource_mut::<Events<ClientInstanceReport>>().send(ClientInstanceReport::RequestConnectToken(game_id));
                set_and_apply_state(w, ClientAppState::Client);
            }
            Self::End => {
                if *w.resource::<State<ClientAppState>>().get() != ClientAppState::Game {
                    tracing::warn!("ignoring command to end game, client is not in a game");
                    return;
                }
                let game_id = w.resource::<ClientFwConfig>().game_id();
                tracing::debug!("ending game {game_id}");
                w.resource_mut::<Events<ClientInstanceReport>>().send(ClientInstanceReport::Ended(game_id));
                set_and_apply_state(w, ClientAppState::Client);
            }
            Self::Abort => {
                if *w.resource::<State<ClientAppState>>().get() != ClientAppState::Game {
                    tracing::warn!("ignoring command to abort game, client is not in a game");
                    return;
                }
                let game_id = w.resource::<ClientFwConfig>().game_id();
                tracing::warn!("aborting game {game_id}");
                w.resource_mut::<Events<ClientInstanceReport>>().send(ClientInstanceReport::Aborted(game_id));
                set_and_apply_state(w, ClientAppState::Client);
            }
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
