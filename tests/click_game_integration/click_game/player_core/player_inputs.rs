//local shortcuts

//third-party shortcuts

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during Init mode.
#[derive(Debug)]
pub enum PlayerInputInit
{
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during Prep mode.
#[derive(Debug)]
pub enum PlayerInputPrep
{
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during Play mode.
#[derive(Debug)]
pub enum PlayerInputPlay
{
    ClickButton,
    None,
}

//-------------------------------------------------------------------------------------------------------------------

/// Inputs that may come from a player during GameOver mode.
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
    Prep(PlayerInputPrep),
    Play(PlayerInputPlay),
    GameOver(PlayerInputGameOver),
}

//-------------------------------------------------------------------------------------------------------------------
