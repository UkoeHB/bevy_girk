//local shortcuts

//third-party shortcuts
use serde::Deserialize;

//standard shortcuts


//-------------------------------------------------------------------------------------------------------------------

/**!
Parse a JSON-serialized type.

Compatible with [`clap::builder::ValueParser`](https://docs.rs/clap/latest/clap/builder/struct.ValueParser.html).
Useful for passing arguments into a binary via rust, e.g. with
[`std::process::Command`](https://doc.rust-lang.org/std/process/struct.Command.html).

Example:
```no-run
#[derive(Serialize, Deserialize)]
struct MyArgs
{
    val: u32,
}

#[derive(Parser, Debug)]
struct MyCli
{
    #[arg(long, value_parser = parse_json::<MyArgs>)]
    myargs: MyArgs,
}
```
*/
pub fn parse_json<T: for<'de> Deserialize<'de>>(arg: &str) -> Result<T, Box<dyn std::error::Error + Send + Sync + 'static>>
{
    Ok(serde_json::de::from_str::<T>(&arg)?)
}

//-------------------------------------------------------------------------------------------------------------------
