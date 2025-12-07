use helioscope_common::ProbeDataPoint;
use sqlx::{Acquire, SqliteConnection};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::store::db::Database;
use crate::store::errors::StoreError;

const CHANNEL_BUFFER_SIZE: usize = 1000;

/// Command sent to the writer task
pub enum WriteCommand {
    InsertBatch(Vec<ProbeDataPoint>),
    Shutdown,
}

/// Handle for sending write commands to the database writer task
#[derive(Clone)]
pub struct WriterHandle {
    tx: mpsc::Sender<WriteCommand>,
}

impl WriterHandle {
    /// Send a batch of probe data points to be written
    pub async fn insert_batch(&self, data: Vec<ProbeDataPoint>) -> Result<(), StoreError> {
        self.tx
            .send(WriteCommand::InsertBatch(data))
            .await
            .map_err(|_| StoreError::NotInitialized)?;
        Ok(())
    }

    /// Signal the writer task to shut down gracefully
    pub async fn shutdown(&self) -> Result<(), StoreError> {
        self.tx
            .send(WriteCommand::Shutdown)
            .await
            .map_err(|_| StoreError::NotInitialized)?;
        Ok(())
    }
}

/// Database writer service that processes write commands from a channel
pub struct WriterService {
    db: Database,
    rx: mpsc::Receiver<WriteCommand>,
}

impl WriterService {
    /// Create a new writer service and return a handle for sending commands
    pub async fn new(data_dir: &str) -> Result<(Self, WriterHandle), StoreError> {
        let db = Database::new(data_dir).await?;
        let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);

        let service = Self { db, rx };
        let handle = WriterHandle { tx };

        Ok((service, handle))
    }

    /// Run the writer task, processing commands until shutdown
    pub async fn run(mut self) {
        info!("Starting database writer service");

        while let Some(cmd) = self.rx.recv().await {
            match cmd {
                WriteCommand::InsertBatch(data) => {
                    if let Err(e) = self.handle_insert_batch(data).await {
                        error!("Failed to insert batch: {}", e);
                    }
                }
                WriteCommand::Shutdown => {
                    info!("Received shutdown command");
                    break;
                }
            }
        }

        // Drain remaining commands before shutting down
        self.drain_remaining_commands().await;

        if let Err(e) = self.db.close().await {
            error!("Error closing database: {}", e);
        }

        info!("Database writer service stopped");
    }

    async fn handle_insert_batch(&mut self, data: Vec<ProbeDataPoint>) -> Result<(), StoreError> {
        let count = data.len();
        debug!("Processing insert batch of {} items", count);

        let inserted = insert_batch(self.db.conn(), &data).await?;

        if inserted != count as u64 {
            warn!(
                "Expected to insert {} items but inserted {}",
                count, inserted
            );
        }

        Ok(())
    }

    async fn drain_remaining_commands(&mut self) {
        let mut drained = 0;

        while let Ok(cmd) = self.rx.try_recv() {
            match cmd {
                WriteCommand::InsertBatch(data) => {
                    if let Err(e) = self.handle_insert_batch(data).await {
                        error!("Failed to insert batch during drain: {}", e);
                    }
                    drained += 1;
                }
                WriteCommand::Shutdown => break,
            }
        }

        if drained > 0 {
            info!("Drained {} remaining commands before shutdown", drained);
        }
    }
}

/// Insert multiple probe data points in a transaction
async fn insert_batch(
    conn: &mut SqliteConnection,
    data_points: &[ProbeDataPoint],
) -> Result<u64, StoreError> {
    if data_points.is_empty() {
        return Ok(0);
    }

    debug!("Inserting batch of {} probe data points", data_points.len());

    let mut tx = conn.begin().await?;
    let mut count = 0u64;

    for data in data_points {
        sqlx::query(
            r#"
            INSERT INTO probe_data (node_id, timestamp, probe_type, probe_name, probe_value)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(&data.node_id)
        .bind(&data.timestamp)
        .bind(&data.probe_type)
        .bind(&data.probe_name)
        .bind(&data.probe_value)
        .execute(&mut *tx)
        .await?;

        count += 1;
    }

    tx.commit().await?;

    debug!("Successfully inserted {} probe data points", count);
    Ok(count)
}
