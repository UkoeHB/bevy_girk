//local shortcuts

//third-party shortcuts
use bincode::Options;
use bytes::Bytes;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

//standard shortcuts
use std::io::Cursor;

//-------------------------------------------------------------------------------------------------------------------

/// Serializes a message.
pub fn ser_msg<T: Serialize>(message_object: &T) -> Vec<u8>
{
    bincode::DefaultOptions::new().serialize(message_object).unwrap()
}

//-------------------------------------------------------------------------------------------------------------------

/// Deserializes a message.
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

/// Partially deserializes a message from a `Bytes` instance.
///
/// Bytes used when deserializing to `T` will be cropped from the message.
pub fn deser_bytes_partial<T: for<'de> Deserialize<'de>>(message: &mut Bytes) -> Option<T>
{
    let mut cursor = Cursor::new(&message[..]);
    let Ok(value) = bincode::DefaultOptions::new().deserialize_from(&mut cursor) else { return None; };
    let _ = message.split_to(cursor.position() as usize);

    Some(value)
}

//-------------------------------------------------------------------------------------------------------------------
