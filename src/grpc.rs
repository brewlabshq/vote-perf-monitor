use {
    crate::config::Config,
    futures::StreamExt,
    std::sync::Arc,
    tokio::{
        sync::{broadcast::Sender, mpsc::UnboundedSender},
        task::JoinHandle,
    },
    yellowstone_grpc_client::{ClientTlsConfig, GeyserGrpcBuilder, GeyserGrpcClient},
    yellowstone_grpc_proto::geyser::{
        CommitmentLevel, SubscribeRequest, SubscribeRequestFilterBlocks,
        SubscribeRequestFilterTransactions, SubscribeUpdateBlock, SubscribeUpdateTransaction,
        subscribe_update::UpdateOneof,
    },
};
pub fn create_grpc_client() -> Result<GeyserGrpcBuilder, anyhow::Error> {
    let config = Config::load_from_env().expect("Error: unable to load config");

    let client = GeyserGrpcClient::build_from_shared(config.grpc_url.clone())?
        .x_token(config.x_token.clone())?
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

pub async fn handle_grpc_streams(
    block_sender: Arc<Sender<SubscribeUpdateBlock>>,
    tx_sender: Arc<Sender<SubscribeUpdateTransaction>>,
    shutdown_sender: Arc<Sender<bool>>,
) -> Result<JoinHandle<()>, anyhow::Error> {
    let grpc_client = create_grpc_client()?;

    let subscribe_request: SubscribeRequest = grpc_client_subscribe_request();

    let (mut _subscribe, mut stream) = match grpc_client.connect().await {
        Ok(mut r) => match r.subscribe_with_request(Some(subscribe_request)).await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("Error: unable to subscribe with request : {:?}", e);
                return Err(anyhow::Error::msg("Error: unable to subscribe"));
            }
        },
        Err(e) => {
            tracing::error!("Error: unable to build grpc stream connection {:?}", e);
            return Err(anyhow::Error::msg(
                "Error: unable to connect to the grpc stream",
            ));
        }
    };

    let mut shutdown_rx = shutdown_sender.subscribe();

    let jh = tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                tracing::info!("Shutdown signal received");
                    break;
                }

                Some(msg) = stream.next() => {

                    match msg {
                                 Ok(message) => match message.update_oneof {
                    Some(UpdateOneof::Block(message)) => {
                        if let Err(e) = block_sender.send(message) {
                            tracing::error!("Error: unable to send block update {:?}", e);
                            continue;
                        }
                    }
                    Some(UpdateOneof::Transaction(message)) => {
                        if let Err(e) = tx_sender.send(message) {
                            tracing::error!("Error: unable to send transcation update {:?}", e);
                            continue;
                        }
                    }
                    _ => {}
                }
                        Err(e)=>{
                            tracing::error!("Error: unable to handle subscribe message: {:?}",e)
                        }
                    }
                }

                else => {
                    tracing::info!("Both shutdown and stream closed");
                    break;
                }
            }
        }
    });

    Ok(jh)
}
