use tide::Request;
use tide::Response;
use tide::prelude::*;
use std::env;
use std::string::String;
use mysql::PooledConn;
use mysql::Value::NULL;

#[async_std::main]
async fn main() -> tide::Result<()>
{
    let args: Vec<String> = env::args().collect();
    let address = &args[1];
    let listen = format!("{}", address);
    let mut server = tide::new();
    tide::log::start();
 
    server.listen(listen).await?;
 
    Ok(())
}
