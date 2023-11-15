//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");

//-------------------------------------------------------------------------------------------------------------------
//-------------------------------------------------------------------------------------------------------------------

/// SERVER
pub type HostUserServer       = bevy_simplenet::Server<HostUserChannel>;
pub type HostUserServerReport = bevy_simplenet::ServerReport<()>;
pub type HostUserServerEvent  = bevy_simplenet::ServerEventFrom<HostUserChannel>;

/// server factory
pub fn host_user_server_factory() ->  bevy_simplenet::ServerFactory<HostUserChannel>
{
    bevy_simplenet::ServerFactory::<HostUserChannel>::new(PACKAGE_VERSION)
}

//-------------------------------------------------------------------------------------------------------------------
