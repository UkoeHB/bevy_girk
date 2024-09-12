//local shortcuts

//third-party shortcuts
use bevy::prelude::*;
use wasm_timer::{SystemTime, UNIX_EPOCH};

//standard shortcuts
use std::{fmt::Debug, sync::{Arc, Mutex}};

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn get_systime() -> Duration
{
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default()
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

struct RunningLocalGame
{
    game: GameInstance,
    reports: IoReceiver<GameInstanceReport>,
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Returns a game id if something went wrong and we need to shut it down.
fn handle_game_instance_report(w: &mut World, report: GameInstanceReport) -> Option<u64>
{
    match report
    {
        GameInstanceReport::GameStart(game_id, start_report) =>
        {
            if *w.resource::<State<ClientInstanceState>>().get() != ClientInstanceState::Game
            {
                tracing::warn!("ignoring game start report for local game {}; client is not in \
                    ClientInstanceState::Game", game_id);
                return Some(game_id);
            }

            let Some(start_info) = start_report.start_infos.pop() else {
                tracing::error!("ignoring game start report for local game {}; report is missing start info",
                    game_id);
                return Some(game_id);
            };

            let Some(meta) = &report.memory_meta else {
                tracing::error!("ignoring game start report for local game; in-memory meta is missing for \
                    setting up renet client");
                return Some(game_id);
            };
            let Ok(token) = meta.new_connect_token(0, get_systime(), start_info.client_id) else {
                tracing::error!("ignoring game start report for local game {}; failed producing in-memory \
                    connect token", game_id);
                return Some(game_id);
            };

            tracing::info!("setting up local-player game {game_id}");
            w.resource_scope(|w: &mut World, mut factory: Mut<ClientFactory>| {
                factory.setup_game(w, token, start_info);
            });

            None
        }
        GameInstanceReport::GameOver(game_id, end_report) =>
        {
            tracing::info!("local-player game {game_id} ended");
            w.resource_mut::<LocalGameManager>().try_set_last_game(
                game_id,
                LocalGameReport::End{ game_id, report: end_report }
            );
            // NOTE: Do not discard the game yet, it may still need to communicate with the client.
            None
        }
        GameInstanceReport::GameAborted(game_id) =>
        {
            tracing::warn!("local-player game server aborted, force-closing game client");
            w.resource_mut::<LocalGameManager>().try_set_last_game(
                game_id,
                LocalGameReport::Aborted{ game_id }
            );
            w.resource_mut::<LocalGameManager>().discard_current_game();
            ClientInstanceCommand::Abort.apply(w);

            None
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

fn monitor_local_game_reports(w: &mut World)
{
    let reports = w.resource_mut::<LocalGameManager>().reports();
    for report in reports
    {
        if let Some(bad_game) = handle_game_instance_report(w, report) {
            if !w.resource_mut::<LocalGameManager>().try_abort_game(bad_game) { continue }
            tracing::warn!("local-player game server aborted, force-closing game client");
            ClientInstanceCommand::Abort.apply(w);
        }
    }
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Final cleanup mechanism to capture all cases where we don't discard the game server early.
fn try_shut_down_local_game(mut manager: ResMut<LocalGameManager>)
{
    manager.discard_current_game();
}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Reports that can be emitted by local-player game servers.
///
/// See [`LocalGameManager::take_report`].
#[derive(Debug, Clone)]
pub enum LocalGameReport
{
    /// Emitted whenever a [`GameOverReport`] is produced by a local-player game.
    End{
        game_id: u64,
        report: GameOverReport,
    },
    /// Emitted when the game server was aborted.
    Aborted{
        game_id: u64
    }
}

//-------------------------------------------------------------------------------------------------------------------

/// Resource that constructs and monitors ongoing local-player games.
///
/// Inserted to the app via [`ClientInstancePlugin`].
#[derive(Resource)]
pub struct LocalGameManager
{
    // TODO: This currently only launches games in a thread on the same app. It may be useful to allow configuring
    // child process (or webworker) launchers.
    launcher: Option<GameInstanceLauncherLocal>,

    current_game: Option<RunningLocalGame>,

    /// The result from the last local-player game.
    last_game: Option<LocalGameReport>,
}

impl LocalGameManager
{
    pub(crate) fn new(factory: Option<GameFactory>) -> Self
    {
        Self{
            launcher: factory.map(|f| GameInstanceLauncherLocal::new(factory)),
            current_game: None,
            last_game: None,
        }
    }

    pub(crate) fn launch(&mut self, launch_pack: GameLaunchPack)
    {
        let launcher = self.launcher.as_ref().expect("local games can only be started if a GameFactory for local games is \
            specified in ClientInstancePlugin");
        let (game_report_sender, game_report_receiver) = new_io_channel::<GameInstanceReport>();
        let game_instance = launcher.launch(true, launch_pack, game_report_sender);

        if let Some(current) = self.current_game {
            tracing::error!("force-closing current local-player game {} because a new game {} is being launched",
                current.game.id(), game_instance.id());
            current.game.send_command(GameInstanceCommand::Abort);
        }

        self.current_game = Some(RunningLocalGame{ game: game_instance, reports: game_report_receiver });
    }

    /// Takes the result report for the last local-player game that finished.
    ///
    /// Note that the report may appear while in state [`ClientInstanceState::Game`]. It is recommended
    /// to look for reports in [`OnEnter(ClientInstanceState::Client`)].
    pub fn take_report(&mut self) -> Option<LocalGameReport>
    {
        self.last_game.take()
    }

    /// Gets the result report for the last local-player game that finished.
    pub fn get_report(&self) -> Option<&LocalGameReport>
    {
        self.last_game.as_ref()
    }

    /// Returns `true` if a local-player game is running.
    ///
    /// If the game ended successfully, then [`Self::get_report`] will return a value.
    pub fn is_running(&self) -> bool
    {
        self.current_game.is_some()
    }

    fn try_set_last_game(&mut self, game_id: u64, last: LocalGameReport)
    {
        if let Some(prev) = self.last_game
        {
            // We add this check in case a game-over report is emitted and then the game has to abort while sending
            // game-over information to clients.
            // - Note that if local games are given the same `game_id` every time and LocalGameReports are only
            //   sometimes taken, then this can cause game over reports to be lost.
            if prev.game_id == game_id
            {
                tracing::warn!("ignoring new LocalGameReport in LocalGameManager for game {}; previous value was \
                    not extracted", game_id, prev.game_id);
                return;
            }
            tracing::warn!("overwriting LocalGameReport in LocalGameManager for game {}; previous value for game \
                {} was not extracted", game_id, prev.game_id);
        }
        self.last_game = Some(last);
    }

    fn reports(&mut self) -> Vec<GameInstanceReport>
    {
        let mut reports = Vec::default();
        if let Some(game) = self.current_game {
            while let Some(report) = game.reports.try_recv()
            {
                reports.push(report);
            }
        }
        reports
    }

    /// Returns `true` if a game was aborted.
    fn try_abort_game(&mut self, game_id: u64) -> bool
    {
        let Some(current) = self.current_game else { return false };
        if current.game.id() != game_id { return false }

        tracing::error!("force-closing local-player game {} because of a previous error", current.game.id());
        current.game.send_command(GameInstanceCommand::Abort);
        self.last_game = LocalGameReport::Aborted{ game_id };
        self.current_game = None;

        true
    }

    fn discard_current_game(&mut self)
    {
        let Some(current) = self.current_game else { return };
        current.game.send_command(GameInstanceCommand::Abort);
        if self.last_game.is_none() {
            self.last_game = LocalGameReport::Aborted{ game_id: current.game.id() };
        }
        self.current_game = None;
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub(crate) struct LocalGamePlugin
{
    pub(crate) local_factory: Arc<Mutex<Option<GameFactory>>>,
}

impl Plugin for LocalGamePlugin
{
    fn build(&self, app: &mut App)
    {
        let local_factory = self.local_factory
            .lock()
            .expect("LocalGamePlugin should only be built once")
            .take();

        app
            .insert_resource(LocalGameManager::new(local_factory))
            .add_systems(First, monitor_local_game_reports.run_if(in_state(ClientInstanceState::Game)))
            // TODO: This assumes local-player games cannot be paused and resumed. Consider making it more
            // sophisticated.
            .add_systems(OnExit(ClientInstanceState::Game), try_shut_down_local_game);
    }
}

//-------------------------------------------------------------------------------------------------------------------
