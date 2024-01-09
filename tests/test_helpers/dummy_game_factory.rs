//local shortcuts
use crate::test_helpers::*;
use bevy_girk_client_fw::*;
use bevy_girk_game_fw::*;
use bevy_girk_game_instance::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize)]
pub struct DummyGameConfig
{
    pub ticks_per_sec       : Ticks,
    pub game_duration_ticks : Ticks,
}

//-------------------------------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct DummyGameFactory;

impl GameFactoryImpl for DummyGameFactory
{
    fn new_game(&self, app: &mut App, launch_pack: GameLaunchPack) -> Result<GameStartReport, ()>
    {
        // extract factory config
        let Some(config) = deser_msg::<DummyGameConfig>(&launch_pack.game_init_data) else { return Err(()); };

        // get player ids
        let player_ids: Vec<u128> = launch_pack.client_init_data.iter().map(|m| m.user_id).collect();

        // prepare message channels
        let (client_packet_sender, client_packet_receiver)      = new_channel::<ClientPacket>();
        let (game_packet_sender, game_packet_receiver)          = new_channel::<GamePacket>();
        let (_client_fw_comand_sender, client_fw_comand_reader) = new_channel::<ClientFwCommand>();

        // make the client ready
        client_packet_sender.send(
                ClientPacket{
                        client_id   : 0 as ClientIdType,
                        send_policy : SendOrdered.into(),
                        message     : bytes::Bytes::from(ser_msg(&ClientMessage{
                                message: AimedMsg::<_, ()>::Fw(ClientFwRequest::SetInitProgress(1.0))
                            }))
                    }
            ).unwrap();

        // prepare app
        app
            //bevy plugins
            .add_plugins(bevy::time::TimePlugin)
            //setup game framework
            .insert_resource(GameFwConfig::new( config.ticks_per_sec, Ticks(1), Ticks(0) ))
            .insert_resource(prepare_player_client_contexts(player_ids.len()))
            .insert_resource(client_packet_receiver)
            .insert_resource(game_packet_sender)
            //setup client framework
            .insert_resource(
                ClientFwConfig::new( config.ticks_per_sec, 0 as ClientIdType )
            )
            .insert_resource(game_packet_receiver)
            .insert_resource(client_packet_sender)
            .insert_resource(client_fw_comand_reader)
            //setup game core
            .insert_resource(DummyGameDurationConfig{ max_ticks: config.game_duration_ticks })
            //add game framework
            .add_plugins(GameFwPlugin)
            //add client framework
            .add_plugins(ClientFwPlugin)
            //add game
            .add_plugins(DummyGameCorePlugin)
            //add client
            .add_plugins(DummyClientCorePlugin)
            //configure execution flow
            .configure_sets(Update, (GameFwSet, ClientFwSet).chain());

        // prepare dummy token meta
        let native_meta = Some(ConnectMetaNative::dummy());

        // prepare dummy connect infos
        let start_infos: Vec<GameStartInfo> = player_ids.iter().map(|id| GameStartInfo::new_from_id(*id)).collect();

        // prepare dummy game start report
        let game_start_report = GameStartReport{ native_meta, wasm_meta: None, start_infos };

        Ok(game_start_report)
    }
}

//-------------------------------------------------------------------------------------------------------------------
