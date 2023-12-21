//local shortcuts

//third-party shortcuts
use serde::{Deserialize, Serialize};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum GameInstanceCommand
{
    Abort
}

//-------------------------------------------------------------------------------------------------------------------
