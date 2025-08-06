use {
    crate::config::{self, Config},
    std::{any, sync::Arc},
    tokio::{
        sync::{broadcast::Sender, mpsc::UnboundedSender},
        task::JoinHandle,
    },
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

pub async fn process_vote_tx(
    tx: SubscribeUpdateTransaction,
    config: Arc<Config>,
) -> Result<(), anyhow::Error> {
    Ok(())
}

pub async fn process_block(
    block: SubscribeUpdateBlock,
    config: Arc<Config>,
) -> Result<(), anyhow::Error> {
    Ok(())
}
