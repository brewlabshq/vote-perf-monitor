use {
    crate::config::Config,
    std::sync::Arc,
    tokio::{sync::broadcast::Sender, task::JoinHandle},
    yellowstone_grpc_proto::geyser::{SubscribeUpdateBlock, SubscribeUpdateTransaction},
};

pub async fn handle_block_updates(
    block_sender: Arc<Sender<SubscribeUpdateBlock>>,
    shutdown_sender: Arc<Sender<bool>>,
    config: Arc<Config>,
) -> Result<JoinHandle<()>, anyhow::Error> {
    let jh = tokio::spawn(async move {
        let mut shutdown_rx = shutdown_sender.subscribe();
        let mut block_rx = block_sender.subscribe();
        loop {
            tokio::select! {
                _= shutdown_rx.recv()=>{
                    tracing::info!("Shutting down channels");
                    break;
                }

                Ok(msg) = block_rx.recv()=>{
               if let Err(e) =  process_block(msg, config.clone()).await{
                    tracing::error!("Error: unable to process vote: {:?}",e);
                   };
                },

                else => {
                    tracing::info!("Both shutdown and stream closed");
                    break;
                }

            }
        }
    });
    Ok(jh)
}

pub async fn handle_tx_updates(
    tx_sender: Arc<Sender<SubscribeUpdateTransaction>>,
    shutdown_sender: Arc<Sender<bool>>,
    config: Arc<Config>,
) -> Result<JoinHandle<()>, anyhow::Error> {
    let jh = tokio::spawn(async move {
        let mut shutdown_rx = shutdown_sender.subscribe();
        let mut tx_rx = tx_sender.subscribe();
        loop {
            tokio::select! {
                _= shutdown_rx.recv()=>{
                    tracing::info!("Shutting down channels");
                    break;
                }

                Ok(msg) = tx_rx.recv()=>{
                   if let Err(e) =  process_vote_tx(msg, config.clone()).await{
                    tracing::error!("Error: unable to process vote: {:?}",e);
                   };

                },

                else => {
                    tracing::info!("Both shutdown and stream closed");
                    break;
                }

            }
        }
    });
    Ok(jh)
}

pub const VOTE_PROGRAM_ID: [u8; 32] = [
    7, 97, 72, 29, 53, 116, 116, 187, 124, 77, 118, 36, 235, 211, 189, 179, 216, 53, 94, 115, 209,
    16, 67, 252, 13, 163, 83, 128, 0, 0, 0, 0,
];

pub async fn process_vote_tx(
    tx: SubscribeUpdateTransaction,
    config: Arc<Config>,
) -> Result<(), anyhow::Error> {
    // tracing::info!("{:?}", tx);
    Ok(())
}

pub async fn process_block(
    block: SubscribeUpdateBlock,
    config: Arc<Config>,
) -> Result<(), anyhow::Error> {
    // for i in block.transactions.iter() {
    //     // i.hash
    //     if i.is_vote {
    //         solana_signature::Signature(
    //             i.transaction.clone().unwrap().signatures[0].splice(range, replace_with),
    //         );
    //         tracing::info!("{:?}", i.transaction.clone().unwrap().signatures[0]);

    //         // i.transaction.unwrap().message.unwrap().instructions
    //     }
    // }

    for tx_meta in &block.transactions {
        if !tx_meta.is_vote {
            continue;
        }

        // Safely access the inner transaction
        match &tx_meta.transaction {
            Some(tx) => {
                // Get the first signature if present
                if let Some(sig) = tx.signatures.get(0) {
                    // Signature implements Display -> Base58 string
                    tracing::info!("vote tx sig={:?}", sig);

                    // If you ever need the raw 64 bytes:
                    // let sig_bytes: &[u8; 64] = sig.as_ref();

                    // Or as Base58 manually:
                    let sig_b58 = bs58::encode(sig[..64].as_ref()).into_string();

                    tracing::info!("{:?}", sig_b58);
                } else {
                    tracing::warn!("vote tx has no signatures; hash:{:?}", tx_meta.signature);
                }
            }
            None => {
                tracing::warn!(
                    "missing transaction for vote entry; hash:{:?}",
                    tx_meta.signature
                );
            }
        }
    }
    Ok(())
}
