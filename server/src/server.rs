use crate::server::zkp_auth::{
    AuthenticationAnswerRequest, AuthenticationAnswerResponse, AuthenticationChallengeRequest,
    AuthenticationChallengeResponse, RegisterRequest, RegisterResponse,
};
use num_bigint::BigInt;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub mod zkp_auth {
    tonic::include_proto!("zkp_auth");
}

#[derive(Default)]
pub struct ZkpServer {
    inner: Mutex<ZkpServerInner>,
}

// An in-memory backend. A real implementation would use a database
#[derive(Default)]
struct ZkpServerInner {
    registrations: HashMap<String, Registration>,
    login_challenges: HashMap<String, LoginChallenge>,
    active_sessions: HashMap<String, Session>,
    auth_id_to_user: HashMap<String, String>,
}

struct Registration {
    y1: BigInt,
    y2: BigInt,
    g: BigInt,
    h: BigInt,
}

struct LoginChallenge {
    r1: BigInt,
    r2: BigInt,
    c: BigInt,
}

#[allow(dead_code)]
struct Session {
    user: String,
    session_id: String,
}

#[tonic::async_trait]
impl zkp_auth::auth_server::Auth for ZkpServer {
    async fn register(
        &self,
        request: Request<RegisterRequest>,
    ) -> Result<Response<RegisterResponse>, Status> {
        let request = request.into_inner();
        let registration = Registration {
            y1: BigInt::from_signed_bytes_be(&request.y1),
            y2: BigInt::from_signed_bytes_be(&request.y2),
            g: BigInt::from_signed_bytes_be(&request.g),
            h: BigInt::from_signed_bytes_be(&request.h),
        };

        let mut lock = self.inner.lock().await;
        if lock.registrations.contains_key(&request.user) {
            return Err(Status::already_exists("User already registered"));
        }

        lock.registrations.insert(request.user, registration);
        Ok(Response::new(RegisterResponse {}))
    }

    async fn create_authentication_challenge(
        &self,
        request: Request<AuthenticationChallengeRequest>,
    ) -> Result<Response<AuthenticationChallengeResponse>, Status> {
        // Generate a random c
        let request = request.into_inner();
        let c: u8 = rand::random::<u8>() % 64; // Limit randomness to 64 max to prevent overflow when computing s
        let c = BigInt::from(c);

        // Ensure the same user isn't already trying to login
        let mut lock = self.inner.lock().await;
        if lock.login_challenges.contains_key(&request.user) {
            return Err(Status::already_exists("User already attempting to login"));
        }

        lock.login_challenges.insert(
            request.user.clone(),
            LoginChallenge {
                r1: BigInt::from_signed_bytes_be(&request.r1),
                r2: BigInt::from_signed_bytes_be(&request.r2),
                c: c.clone(),
            },
        );

        let auth_id = uuid::Uuid::new_v4().to_string();

        lock.auth_id_to_user.insert(auth_id.clone(), request.user);

        Ok(Response::new(AuthenticationChallengeResponse {
            c: c.to_signed_bytes_be(),
            auth_id,
        }))
    }

    async fn verify_authentication(
        &self,
        request: Request<AuthenticationAnswerRequest>,
    ) -> Result<Response<AuthenticationAnswerResponse>, Status> {
        let request = request.into_inner();
        let mut lock = self.inner.lock().await;
        // Verify that r1 = g^s * y1^c AND r2 = h^s * y2^c
        let user_name = lock
            .auth_id_to_user
            .remove(&request.auth_id)
            .ok_or(Status::not_found(
                "No login challenge found for the provided auth_id",
            ))?;
        let login = lock
            .login_challenges
            .get(&user_name)
            .ok_or(Status::not_found("No login challenge found"))?;
        let registration = lock
            .registrations
            .get(&user_name)
            .ok_or(Status::not_found("No registration found"))?;
        // Verify that r1 = g^s * y1^c AND r2 = h^s * y2^c
        let g = registration.g.clone();
        let h = registration.h.clone();
        let y1 = registration.y1.clone();
        let y2 = registration.y2.clone();
        let s = BigInt::from_signed_bytes_be(&request.s)
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid s value"))?;
        let c = login
            .c
            .clone()
            .try_into()
            .map_err(|_| Status::invalid_argument("Invalid c value"))?;

        let rhs1 = g.pow(s) * y1.pow(c);
        let rhs2 = h.pow(s) * y2.pow(c);

        if rhs1 != login.r1 || rhs2 != login.r2 {
            return Err(Status::invalid_argument("Invalid login credentials"));
        }

        let session_id = uuid::Uuid::new_v4().to_string();

        lock.active_sessions.insert(
            session_id.clone(),
            Session {
                user: user_name,
                session_id: session_id.clone(),
            },
        );

        Ok(Response::new(AuthenticationAnswerResponse { session_id }))
    }
}
