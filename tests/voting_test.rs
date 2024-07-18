use assertables::{assert_contains, assert_contains_as_result};

use domain::{
    move_future_meet_up_to_voting, show_voting, store_votes, submit_paper, Location, MeetUpState,
};
use shared::utc_now;
use tests::{build_gateway, build_paper_with_user, create_meet_up, create_random_user};

#[::tokio::test]
async fn show_voting_without_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let err = show_voting(&gateway, &gateway, &gateway, &user.id)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn show_voting_with_invalid_meet_up_state() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let _ = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    let err = show_voting(&gateway, &gateway, &gateway, &user.id)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: CallForPapers", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn show_voting_without_papers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::Voting,
    )
    .await?;
    let (voting_meet_up, papers) = show_voting(&gateway, &gateway, &gateway, &user.id).await?;
    assert_eq!(meet_up, voting_meet_up);
    assert!(papers.is_empty());
    Ok(())
}

#[::tokio::test]
async fn show_voting_with_papers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper_1 = build_paper_with_user(user.id.clone());
    let paper_2 = build_paper_with_user(user.id.clone());
    let mut meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper_1.clone()).await?;
    submit_paper(&gateway, &gateway, paper_2.clone()).await?;
    move_future_meet_up_to_voting(&gateway).await?;
    meet_up.state = MeetUpState::Voting;
    let (voting_meet_up, papers) = show_voting(&gateway, &gateway, &gateway, &user.id).await?;
    assert_eq!(meet_up, voting_meet_up);
    assert_contains!(papers, &paper_1);
    assert_contains!(papers, &paper_2);
    Ok(())
}

#[::tokio::test]
async fn store_and_show_voting() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper_1 = build_paper_with_user(user.id.clone());
    let paper_2 = build_paper_with_user(user.id.clone());
    let mut meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper_1.clone()).await?;
    submit_paper(&gateway, &gateway, paper_2.clone()).await?;
    move_future_meet_up_to_voting(&gateway).await?;
    meet_up.state = MeetUpState::Voting;
    store_votes(&gateway, &gateway, &user.id, vec![paper_2.id, paper_1.id]).await?;
    let (voting_meet_up, papers) = show_voting(&gateway, &gateway, &gateway, &user.id).await?;
    assert_eq!(meet_up, voting_meet_up);
    assert_eq!(vec![paper_2, paper_1], papers);
    Ok(())
}
