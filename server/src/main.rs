use std::error::Error;

pub mod server;

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ZKP Server Example")]
struct Opt {
    #[structopt(short, long)]
    bind: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opt = Opt::from_args();

    let address = opts.bind.parse().unwrap();
    let zkp_service = server::ZkpServer::default();
    let zkp_server = server::zkp_auth::auth_server::AuthServer::new(zkp_service)
        .max_decoding_message_size(1024 * 1024 * 1024) // To allow large messages
        .max_encoding_message_size(1024 * 1024 * 1024);

    tonic::transport::Server::builder()
        .add_service(zkp_server)
        .serve(address)
        .await?;
    Ok(())
}
