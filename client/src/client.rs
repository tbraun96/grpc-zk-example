pub use zkp_auth::{auth_client::AuthClient, *};
pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}
