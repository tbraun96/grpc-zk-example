use crate::client::{AuthenticationAnswerRequest, AuthenticationChallengeRequest, RegisterRequest};
use num_bigint::BigInt;
use std::error::Error;
use structopt::StructOpt;

pub mod client;

#[derive(Debug, StructOpt)]
#[structopt(name = "ZKP Client Example")]
struct Opt {
    #[structopt(short, long)]
    /// Should take the form: "http://ip-addr:port"
    server: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let opts: Opt = Opt::from_args();
    let mut client = client::AuthClient::connect(opts.server).await?;

    let user = uuid::Uuid::new_v4().to_string();

    // Begin by registering
    let g = BigInt::from(1); // Public
    let h = BigInt::from(2); // Public
    let q = BigInt::from(i64::MAX); // Public

    let secret: u32 = 3; // Private

    let y1 = g.pow(secret); // g^x
    let y2 = h.pow(secret); // h^x

    // Step 1: Register to server
    let _response = client
        .register(RegisterRequest {
            user: user.clone(),
            y1: y1.to_signed_bytes_be(),
            y2: y2.to_signed_bytes_be(),
            g: g.to_signed_bytes_be(),
            h: h.to_signed_bytes_be(),
        })
        .await?;

    // Step 2: Begin login
    let random_k = rand::random::<u32>();

    // Commit (r1, r2) = (g^k, h^k)
    let r1 = g.pow(random_k);
    let r2 = h.pow(random_k);

    let response = client
        .create_authentication_challenge(AuthenticationChallengeRequest {
            user: user.clone(),
            r1: r1.to_signed_bytes_be(),
            r2: r2.to_signed_bytes_be(),
        })
        .await?
        .into_inner();

    let random_c: BigInt = BigInt::from_signed_bytes_be(&response.c);
    let auth_id = response.auth_id;

    // Step 3: compute s = k -cx (mod q)
    let s = BigInt::from(random_k) - (random_c * BigInt::from(secret));
    let s_mod_q = s % q;
    let _response = client
        .verify_authentication(AuthenticationAnswerRequest {
            auth_id,
            s: s_mod_q.to_signed_bytes_be(),
        })
        .await?;

    // Successfully logged-in
    Ok(())
}
