//local shortcuts
use crate::*;
use bevy_girk_game_fw::*;
use bevy_girk_utils::*;

//third-party shortcuts
use bevy::prelude::*;
use bevy_kot_utils::*;
use bevy_replicon::prelude::*;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// Dummy system, does nothing.
fn dummy() {}

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_game_app_framework(
    game_app            : &mut App,
    game_fw_config      : GameFWConfig,
    game_fw_initializer : GameFWInitializer,
){
    // prepare message channels
    let (game_packet_sender, game_packet_receiver)     = new_channel::<GamePacket>();
    let (client_packet_sender, client_packet_receiver) = new_channel::<ClientPacket>();

    // prepare server app
    game_app
        //setup components
        .add_plugins(GameFWPlugin)
        //game framework
        .insert_resource(game_fw_config)
        .insert_resource(game_fw_initializer)
        .insert_resource(game_packet_sender)
        .insert_resource(game_packet_receiver)
        .insert_resource(client_packet_sender)
        .insert_resource(client_packet_receiver);
}

//-------------------------------------------------------------------------------------------------------------------

pub fn prepare_game_app_replication(game_app: &mut App)
{
    // depends on game framework

    // setup server with bevy_replicon (includes bevy_renet)
    game_app
        // add bevy_replicon server
        .add_plugins(bevy::time::TimePlugin)  //required by bevy_renet
        .add_plugins(
            ReplicationPlugins
                .build()
                .disable::<ClientPlugin>()
                .set( ServerPlugin::new(TickPolicy::EveryFrame) )
        )
        //bracket the game logic with message receiving/sending (game logic is in `Update`)
        .add_systems(PreUpdate,
            receive_client_messages
                .run_if(resource_exists::<RenetServer>())
                .after(bevy_replicon::prelude::ServerSet::Receive)
                .before(GameFWTickSetPrivate::FWStart)
        )
        .add_systems(PostUpdate,
            send_server_messages
                .run_if(resource_exists::<RenetServer>())
                .after(GameFWTickSetPrivate::FWEnd)
                .before(bevy_replicon::prelude::ServerSet::Send)
        )
        //prepare message channels
        .add_server_event_with::<EventConfig<GamePacket, SendUnreliable>, _, _>(SendPolicy::Unreliable, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendUnordered>, _, _>(SendPolicy::Unordered, dummy, dummy)
        .add_server_event_with::<EventConfig<GamePacket, SendOrdered>, _, _>(SendPolicy::Ordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnreliable>, _, _>(SendPolicy::Unreliable, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendUnordered>, _, _>(SendPolicy::Unordered, dummy, dummy)
        .add_client_event_with::<EventConfig<ClientPacket, SendOrdered>, _, _>(SendPolicy::Ordered, dummy, dummy);
}

//-------------------------------------------------------------------------------------------------------------------
