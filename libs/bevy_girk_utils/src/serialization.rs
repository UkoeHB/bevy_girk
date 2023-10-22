//local shortcuts

//third-party shortcuts
use bincode::Options;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

pub fn ser_msg<T: Serialize>(message_object: &T) -> Vec<u8>
{
    bincode::DefaultOptions::new().serialize(message_object).unwrap()
}

//-------------------------------------------------------------------------------------------------------------------

pub fn deser_msg<'a, T: Deserialize<'a>>(message: &'a [u8]) -> Option<T>
{
    match bincode::DefaultOptions::new().deserialize::<T>(message)
    {
        Ok(result) => Some(result),
        _          => None
    }
}

//-------------------------------------------------------------------------------------------------------------------

pub fn deser_msg_from<R: std::io::Read, T: DeserializeOwned>(reader: R) -> Option<T>
{
    match bincode::DefaultOptions::new().deserialize_from(reader)
    {
        Ok(result) => Some(result),
        _          => None
    }
}

//-------------------------------------------------------------------------------------------------------------------
