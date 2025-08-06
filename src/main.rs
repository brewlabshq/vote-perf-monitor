use {
    crate::grpc::handle_grpc_streams,
    futures::future::join_all,
    std::sync::Arc,
    tokio::sync::mpsc,
    tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt},
    yellowstone_grpc_proto::geyser::{SubscribeUpdateBlock, SubscribeUpdateTransaction},
};

mod config;
mod grpc;

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

    let (block_sender, _block_receiver) = mpsc::unbounded_channel::<SubscribeUpdateBlock>();
    let (tx_sender, _tx_receiver) = mpsc::unbounded_channel::<SubscribeUpdateTransaction>();

    let tx_sender_arc = Arc::new(tx_sender);
    let block_sender_arc = Arc::new(block_sender);

    let jh_grpc = match handle_grpc_streams(block_sender_arc, tx_sender_arc).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("Error: unable to handle grpc stream {:?}", e);
            return Err(anyhow::Error::msg("Error: unable to handle grpc stream"));
        }
    };

    join_all(vec![jh_grpc]).await;

    Ok(())
}
