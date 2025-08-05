use crate::config::Config;
use yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcBuilder, GeyserGrpcClient};
use yellowstone_grpc_proto::geyser::SubscribeRequest;
use yellowstone_grpc_proto::prelude::*;

pub fn create_grpc_client() -> Result<GeyserGrpcBuilder, anyhow::Error> {
    let config = Config::load_from_env().expect("Error: unable to load config");

    let client = GeyserGrpcClient::build_from_shared(config.grpc_url.clone())?
        .x_token(Some(config.x_token.clone()))?
        .tls_config(ClientTlsConfig::new().with_native_roots())?;

    Ok(client)
}

pub fn grpc_client_subscribe_request() -> SubscribeRequest {
    let config = Config::load_from_env().expect("Error: unable to load config");

    let vote_account = config.vote_account.clone();
    SubscribeRequest {
        transactions: std::collections::HashMap::from([(
            "vote_transactions".to_string(),
            SubscribeRequestFilterTransactions {
                vote: Some(true),
                failed: Some(true),
                signature: None,
                account_include: vec![vote_account.to_string()],
                account_exclude: vec![],
                account_required: vec![],
            },
        )]),
        blocks: std::collections::HashMap::from([(
            "finalized_blocks".to_string(),
            SubscribeRequestFilterBlocks {
                account_include: vec![vote_account.to_string()],
                include_transactions: Some(true),
                include_accounts: Some(false),
                include_entries: Some(false),
            },
        )]),
        commitment: Some(CommitmentLevel::Finalized.into()),
        ..Default::default()
    }
}
