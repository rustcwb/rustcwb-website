use anyhow::Result;
use sqlx::SqlitePool;

mod future_meet_up_gateway;
mod paper_gateway;
mod past_meet_up_gateway;
mod user_gateway;
mod vote_gateway;

pub struct SqliteDatabaseGateway {
    sqlite_pool: SqlitePool,
}

impl SqliteDatabaseGateway {
    pub async fn new(database_url: String) -> Result<Self> {
        let sqlite_pool = SqlitePool::connect(&database_url).await?;
        sqlx::migrate!("./migrations").run(&sqlite_pool).await?;
        Ok(Self { sqlite_pool })
    }
}
