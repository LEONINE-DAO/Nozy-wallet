use crate::error::NozyResult;
use crate::transaction_history::SentTransactionStorage;
use crate::zebra_integration::ZebraClient;
use std::sync::Arc;
use tokio::time::{interval, Duration};

pub struct TransactionConfirmationTracker {
    zebra_client: Arc<ZebraClient>,
    tx_storage: Arc<SentTransactionStorage>,
    check_interval: Duration,
}

impl TransactionConfirmationTracker {
    pub fn new(
        zebra_client: ZebraClient,
        tx_storage: SentTransactionStorage,
        check_interval_secs: u64,
    ) -> Self {
        Self {
            zebra_client: Arc::new(zebra_client),
            tx_storage: Arc::new(tx_storage),
            check_interval: Duration::from_secs(check_interval_secs),
        }
    }

    pub fn start_background_task(&self) -> tokio::task::JoinHandle<()> {
        let zebra_client = Arc::clone(&self.zebra_client);
        let tx_storage = Arc::clone(&self.tx_storage);
        let interval_duration = self.check_interval;

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);

            loop {
                interval.tick().await;

                if let Ok(updated_count) = tx_storage
                    .check_all_pending_transactions(&zebra_client)
                    .await
                {
                    if updated_count > 0 {
                        println!("✅ {} transaction(s) confirmed", updated_count);
                    }
                }

                let _ = tx_storage.update_confirmations(&zebra_client).await;
            }
        })
    }

    pub async fn check_once(&self) -> NozyResult<(usize, usize)> {
        let updated = self
            .tx_storage
            .check_all_pending_transactions(&self.zebra_client)
            .await?;
        let conf_updated = self
            .tx_storage
            .update_confirmations(&self.zebra_client)
            .await?;

        Ok((updated, conf_updated))
    }
}
