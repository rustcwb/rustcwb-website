use domain::{register_event_goer, Location, MeetUpGoersGateway, MeetUpState};
use shared::utc_now;
use tests::{build_gateway, build_paper_with_user, create_meet_up, create_random_user};

#[::tokio::test]
pub async fn register_meet_up_goers_without_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let err = register_event_goer(&gateway, &gateway, &user.id)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
pub async fn register_meet_up_goers_invalid_state() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let _ = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    let err = register_event_goer(&gateway, &gateway, &user.id)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: CallForPapers", err.to_string());
    Ok(())
}

#[::tokio::test]
pub async fn register_meet_up_goers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::Scheduled(paper),
    )
    .await?;
    assert!(
        !gateway
            .is_user_registered_to_meet_up(&user.id, &meet_up.id)
            .await?
    );
    register_event_goer(&gateway, &gateway, &user.id).await?;
    assert!(
        gateway
            .is_user_registered_to_meet_up(&user.id, &meet_up.id)
            .await?
    );
    Ok(())
}
