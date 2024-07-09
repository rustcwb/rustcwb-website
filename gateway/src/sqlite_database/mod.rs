use anyhow::Result;
use sqlx::SqlitePool;

mod meet_up_gateway;
mod meet_up_goers_gateway;
mod paper_gateway;
mod user_gateway;
mod vote_gateway;

pub struct SqliteDatabaseGateway {
    sqlite_pool: SqlitePool,
}

impl SqliteDatabaseGateway {
    pub async fn new(database_url: &str) -> Result<Self> {
        let sqlite_pool = SqlitePool::connect(database_url).await?;
        sqlx::migrate!("./migrations").run(&sqlite_pool).await?;
        Ok(Self { sqlite_pool })
    }
}
