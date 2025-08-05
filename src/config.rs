use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub x_token: String,
    pub grpc_url: String,
    pub vote_account: String,
}

impl Config {
    pub fn load_from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let x_token = env::var("GRPC_X_TOKEN").expect("Error: unable to load grpc x token");
        let grpc_url = env::var("GRPC_URL").expect("Error: unable to load grpc url");
        let vote_account = env::var("VOTE_ACCOUNT").expect("Error: unable to load vote account");

        Ok(Self {
            grpc_url,
            vote_account,
            x_token,
        })
    }
}
