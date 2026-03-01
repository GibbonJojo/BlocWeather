use sqlx::PgPool;
use std::sync::Arc;
use tokio_cron_scheduler::JobScheduler;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub scheduler: Arc<JobScheduler>,
}
