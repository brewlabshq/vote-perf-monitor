use {
    crate::{
        config::Config,
        grpc::handle_grpc_streams,
        vote_tracking::{handle_block_updates, handle_tx_updates},
    },
    futures::future::join_all,
    std::{sync::Arc, time::Duration},
    tokio::{
        sync::{broadcast, mpsc},
        task::JoinHandle,
    },
    tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
    yellowstone_grpc_proto::geyser::{SubscribeUpdateBlock, SubscribeUpdateTransaction},
};

mod config;
mod grpc;
mod vote_tracking;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    let stdout_layer = fmt::layer()
        .with_timer(fmt::time::UtcTime::rfc_3339()) // 2025-06-07T03:37:59Z
        .with_thread_names(true)
        .with_file(true)
        .with_line_number(true)
        .compact(); // concise one-liner
    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(stdout_layer)
        .init();

    let (shutdown_sender, _shutdown_receiver) = broadcast::channel::<bool>(1);

    let (block_sender, _block_receiver) = broadcast::channel::<SubscribeUpdateBlock>(100_000);
    let (tx_sender, _tx_receiver) = broadcast::channel::<SubscribeUpdateTransaction>(100_000);

    let tx_sender_arc = Arc::new(tx_sender);
    let block_sender_arc = Arc::new(block_sender);
    let shutdown_sender_arc = Arc::new(shutdown_sender);
    let config = Config::load_from_env().expect("Error: unable to load config");
    let config = Arc::new(config);

    tracing::info!("Starting grpc stream");
    let jh_grpc: JoinHandle<()> = match handle_grpc_streams(
        block_sender_arc.clone(),
        tx_sender_arc.clone(),
        shutdown_sender_arc.clone(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error: unable to handle grpc stream {:?}", e);
            return Err(anyhow::Error::msg("Error: unable to handle grpc stream"));
        }
    };

    let jh_block_rx = match handle_block_updates(
        block_sender_arc.clone(),
        shutdown_sender_arc.clone(),
        config.clone(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error: uable to init block updates {:?}", e);
            return Err(anyhow::Error::msg("Error: unable to init block updates"));
        }
    };

    let jh_tx_rx = match handle_tx_updates(
        tx_sender_arc.clone(),
        shutdown_sender_arc.clone(),
        config.clone(),
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error: uable to init block updates {:?}", e);
            return Err(anyhow::Error::msg("Error: unable to init block updates"));
        }
    };

    join_all(vec![jh_grpc, jh_block_rx, jh_tx_rx]).await;

    match tokio::signal::ctrl_c().await {
        Ok(r) => {
            tracing::info!("shutdown");

            if shutdown_sender_arc.send(true).is_err() {
                tracing::error!("failed to send shutdown signal");
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        Err(e) => {}
    }
    Ok(())
}
