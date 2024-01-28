use std::error::Error;
use tonic_build;

fn main() -> Result<(), Box<dyn Error>> {
    tonic_build::compile_protos("proto/helloworld.proto")?;
    Ok(())
}