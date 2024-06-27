use anyhow::Result;
use domain::get_future_meet_up;
use gateway::SqliteDatabaseGateway;

async fn build_gateway() -> Result<SqliteDatabaseGateway> {
    SqliteDatabaseGateway::new("sqlite::memory:").await
}

#[::tokio::test]
async fn get_future_meet_up_without_meetup() -> Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = get_future_meet_up(&gateway).await?;
    assert_eq!(None, meet_up);
    Ok(())
}
