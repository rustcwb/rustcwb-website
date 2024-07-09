use assertables::{assert_contains, assert_contains_as_result};
use ulid::Ulid;

use domain::{get_paper, show_call_for_papers, submit_paper, MeetUpState};
use shared::utc_now;
use tests::{build_gateway, build_paper_with_user, create_meet_up, create_random_user};

#[::tokio::test]
async fn show_call_for_papers_without_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let err = show_call_for_papers(&gateway, &gateway, &user)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn show_call_for_papers_with_wrong_state() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let _ = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::Voting,
    )
    .await?;
    let err = show_call_for_papers(&gateway, &gateway, &user)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: Voting", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn show_call_for_papers_without_papers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let meet_up = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    let (show_meet_up, papers, over_the_limit) =
        show_call_for_papers(&gateway, &gateway, &user).await?;
    assert_eq!(meet_up, show_meet_up);
    assert!(papers.is_empty());
    assert!(!over_the_limit);
    Ok(())
}

#[::tokio::test]
async fn show_call_for_papers_with_papers_but_less_than_limit() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let meet_up = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper.clone()).await?;
    let (show_meet_up, papers, over_the_limit) =
        show_call_for_papers(&gateway, &gateway, &user).await?;
    assert_eq!(meet_up, show_meet_up);
    assert_eq!(vec![paper], papers);
    assert!(!over_the_limit);
    Ok(())
}

#[::tokio::test]
async fn show_call_for_papers_with_papers_at_limit() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper_1 = build_paper_with_user(user.id.clone());
    let paper_2 = build_paper_with_user(user.id.clone());
    let meet_up = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper_1.clone()).await?;
    submit_paper(&gateway, &gateway, paper_2.clone()).await?;
    let (show_meet_up, papers, over_the_limit) =
        show_call_for_papers(&gateway, &gateway, &user).await?;
    assert_eq!(meet_up, show_meet_up);
    assert_eq!(2, papers.len());
    assert_contains!(papers, &paper_1);
    assert_contains!(papers, &paper_2);
    assert!(over_the_limit);
    Ok(())
}

#[::tokio::test]
async fn show_call_for_papers_should_only_show_papers_from_correct_user() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user_1 = create_random_user(&gateway).await?;
    let user_2 = create_random_user(&gateway).await?;
    let paper_1 = build_paper_with_user(user_1.id.clone());
    let paper_2 = build_paper_with_user(user_2.id.clone());
    let meet_up = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper_1.clone()).await?;
    submit_paper(&gateway, &gateway, paper_2.clone()).await?;
    let (show_meet_up, papers_1, over_the_limit) =
        show_call_for_papers(&gateway, &gateway, &user_1).await?;
    assert_eq!(meet_up, show_meet_up);
    assert_eq!(vec![paper_1], papers_1);
    assert!(!over_the_limit);
    let (show_meet_up, papers_2, over_the_limit) =
        show_call_for_papers(&gateway, &gateway, &user_2).await?;
    assert_eq!(meet_up, show_meet_up);
    assert_eq!(vec![paper_2], papers_2);
    assert!(!over_the_limit);
    Ok(())
}

#[::tokio::test]
async fn submit_paper_without_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let err = submit_paper(&gateway, &gateway, paper)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn submit_paper_with_invalid_state() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let _ = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::Voting,
    )
    .await?;
    let err = submit_paper(&gateway, &gateway, paper)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: Voting", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn submit_paper_over_limit_per_user() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper_1 = build_paper_with_user(user.id.clone());
    let paper_2 = build_paper_with_user(user.id.clone());
    let paper_3 = build_paper_with_user(user.id.clone());
    let _ = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper_1).await?;
    submit_paper(&gateway, &gateway, paper_2).await?;
    let err = submit_paper(&gateway, &gateway, paper_3)
        .await
        .expect_err("Should error out");
    assert_eq!(
        "More than limit papers per user per meetups. Limit is `2`",
        err.to_string()
    );
    Ok(())
}

#[::tokio::test]
async fn get_invalid_paper() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let invalid_paper_id = Ulid::new();
    let err = get_paper(&gateway, &invalid_paper_id)
        .await
        .expect_err("Should error out");
    assert_eq!(
        format!("Paper not found with id `{invalid_paper_id}`"),
        err.to_string()
    );
    Ok(())
}

#[::tokio::test]
async fn get_paper_should_return_expected_paper() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let _ = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper.clone()).await?;
    assert_eq!(paper, get_paper(&gateway, &paper.id).await?);
    Ok(())
}
