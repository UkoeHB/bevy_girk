//local shortcuts
use crate::test_helpers::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_attributes::*;
use renet2_setup::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct DummyGameConfig
{
    pub ticks_per_sec       : u32,
    pub game_duration_ticks : u32,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DummyClientInit
{
    /// The client's environment type.
    pub env: bevy_simplenet::EnvType,

    /// The client's server-side user id.
    pub user_id: u128,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DummyLaunchPack
{
    pub config: DummyGameConfig,
    pub clients: Vec<DummyClientInit>,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct DummyGameFactory;

impl GameFactoryImpl for DummyGameFactory
{
    type Launch = DummyLaunchPack;

    fn new_game(&self, app: &mut App, _game_id: u64, pack: DummyLaunchPack) -> Result<GameStartReport, String>
    {
        // get player ids
        let player_ids: Vec<u128> = pack.clients.iter().map(|m| m.user_id).collect();

        // prepare message channels
        app.add_event::<ClientPacket>();
        app.add_event::<bevy_replicon::prelude::FromClient<ClientPacket>>();
        app.add_event::<bevy_replicon::prelude::ToClients<GamePacket>>();
        app.add_event::<GamePacket>();

        // make the client ready
        app.world_mut().resource_mut::<Events<FromClient<ClientPacket>>>().send(FromClient{
            client_id: ClientId::SERVER,
            event: ClientPacket{
                send_policy: SendOrdered.into(),
                request: bytes::Bytes::from(ser_msg(&ClientRequestData{
                    req: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                }))
            }
        });

        // prepare app
        app
            //bevy plugins
            .add_plugins(bevy::time::TimePlugin)
            .add_plugins(bevy::state::app::StatesPlugin)
            .add_plugins(bevy::asset::AssetPlugin::default())
            .add_plugins(
                RepliconPlugins
                    .build()
                    .disable::<ClientPlugin>()
                    .set(ServerPlugin{
                        tick_policy: TickPolicy::EveryFrame,
                        visibility_policy: VisibilityPolicy::Whitelist,
                        replicate_after_connect: true,
                        ..Default::default()
                    })
            )
            .add_plugins(VisibilityAttributesPlugin{
                server_id: Some(ClientId::SERVER),
                reconnect_policy: ReconnectPolicy::Reset
            })
            //setup game framework
            .insert_resource(GameFwConfig::new( pack.config.ticks_per_sec, 1, 0 ))
            .insert_resource(prepare_player_client_contexts(player_ids.len() + 1))
            .insert_resource(GameMessageType::new::<()>())
            //setup client framework
            .insert_resource(ClientFwConfig::new( pack.config.ticks_per_sec, 0, ClientId::SERVER ))
            .insert_resource(ClientRequestType::new::<()>())
            //setup game core
            .insert_resource(DummyGameDurationConfig{ max_ticks: pack.config.game_duration_ticks })
            //add game framework
            .add_plugins(GameFwPlugin)
            //add client framework
            .add_plugins(ClientFwPlugin)
            //add game
            .add_plugins(DummyGameCorePlugin)
            //add client
            .add_plugins(DummyClientCorePlugin);

        // prepare dummy token meta
        let mut metas = ConnectMetas::default();
        metas.native = Some(ConnectMetaNative::dummy());

        // prepare dummy connect infos
        let start_infos: Vec<GameStartInfo> = player_ids.iter().map(|id| GameStartInfo::new_from_id(*id)).collect();

        // prepare dummy game start report
        let game_start_report = GameStartReport{ metas, start_infos };

        Ok(game_start_report)
    }
}

//-------------------------------------------------------------------------------------------------------------------
