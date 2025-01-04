//local shortcuts

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during Init state.
#[derive(Debug)]
pub enum PlayerInputInit
{
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during Play state.
#[derive(Debug)]
pub enum PlayerInputPlay
{
    ClickButton,
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during GameOver state.
#[derive(Debug)]
pub enum PlayerInputGameOver
{
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player.
#[derive(Debug)]
pub enum PlayerInput
{
    Init(PlayerInputInit),
    Play(PlayerInputPlay),
    GameOver(PlayerInputGameOver),
}

//-------------------------------------------------------------------------------------------------------------------
