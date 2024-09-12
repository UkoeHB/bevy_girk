//local shortcuts
use bevy_girk_backend_public::*;

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// SERVER
pub type HostUserServer       = bevy_simplenet::Server<HostUserChannel>;
pub type HostUserServerReport = bevy_simplenet::ServerReport<()>;
pub type HostUserServerEvent  = bevy_simplenet::ServerEventFrom<HostUserChannel>;

/// server factory
pub fn host_user_server_factory() ->  bevy_simplenet::ServerFactory<HostUserChannel>
{
    const PACKAGE_VERSION: &str = env!("CARGO_PKG_VERSION");
    bevy_simplenet::ServerFactory::<HostUserChannel>::new(PACKAGE_VERSION)
}

//-------------------------------------------------------------------------------------------------------------------

// Client defined in `backend_public` sub-crate.

//-------------------------------------------------------------------------------------------------------------------
