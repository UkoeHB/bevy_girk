//local shortcuts
use crate::*;

//third-party shortcuts
use enfync::{AdoptOrDefault, Handle};
use bevy_kot_utils::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Launch a game instance in a new thread.
#[derive(Debug)]
pub struct ClientInstanceLauncherLocal
{
    factory: ClientFactory,
}

impl ClientInstanceLauncherLocal
{
    pub fn new(factory: ClientFactory) -> Self
    {
        Self{ factory }
    }
}

impl ClientInstanceLauncherImpl for ClientInstanceLauncherLocal
{
    fn launch(
        &self,
        token                   : ServerConnectToken,
        start_info              : GameStartInfo,
        client_command_receiver : IoReceiver<ClientInstanceCommand>,
        client_report_sender    : IoSender<ClientInstanceReport>,
    ) -> ClientInstance
    {
        // launch game thread
        let game_id = start_info.game_id;
        let factory = self.factory.clone();
        let instance_handle = enfync::builtin::native::CPUHandle::adopt_or_default().spawn(
                async move
                {
                    let Ok(mut app) = client_instance_setup(
                            factory,
                            token,
                            start_info,
                            client_command_receiver,
                            client_report_sender,
                        ) else { return false; };
                    app.run();
                    true
                }
            );

        ClientInstance::new(game_id, instance_handle)
    }
}

//-------------------------------------------------------------------------------------------------------------------
