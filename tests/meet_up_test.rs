use anyhow::Result;
use chrono::NaiveDate;
use ulid::Ulid;

use domain::{
    create_new_meet_up, get_future_meet_up, get_meet_up, get_meet_up_metadata,
    move_future_meet_up_to_done, move_future_meet_up_to_scheduled, move_future_meet_up_to_voting,
    GetPastMeetUpError, MeetUpState, Paper, PaperGateway, Vote, VoteGateway,
};
use tests::{
    assert_meet_up_state, build_gateway, build_paper_with_user, create_meet_up, create_random_user,
};

#[::tokio::test]
async fn get_future_meet_up_without_meetup() -> Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = get_future_meet_up(&gateway).await?;
    assert_eq!(None, meet_up);
    Ok(())
}

#[::tokio::test]
async fn create_and_get_future_meet_up() -> Result<()> {
    let gateway = build_gateway().await?;
    let created_meet_up =
        create_new_meet_up(&gateway, "location".to_string(), "2024-12-12".parse()?).await?;
    let meet_up = get_future_meet_up(&gateway)
        .await?
        .expect("meetup not found");
    assert_eq!(created_meet_up, meet_up);
    assert_eq!("location", meet_up.location);
    assert_eq!("2024-12-12".parse::<NaiveDate>()?, meet_up.date);
    assert_eq!(MeetUpState::CallForPapers, meet_up.state);
    Ok(())
}

#[::tokio::test]
async fn create_and_get_meet_up() -> Result<()> {
    let gateway = build_gateway().await?;
    let created_meet_up =
        create_new_meet_up(&gateway, "location".to_string(), "2024-12-12".parse()?).await?;
    let meet_up = get_meet_up(&gateway, created_meet_up.id).await?;
    assert_eq!(created_meet_up, meet_up);
    assert_eq!("location", meet_up.location);
    assert_eq!("2024-12-12".parse::<NaiveDate>()?, meet_up.date);
    assert_eq!(MeetUpState::CallForPapers, meet_up.state);
    Ok(())
}

#[::tokio::test]
async fn get_not_found_meet_up() -> Result<()> {
    let gateway = build_gateway().await?;
    let id = Ulid::new();
    let err = get_meet_up(&gateway, id)
        .await
        .expect_err("Should error out");
    assert!(matches!(err, GetPastMeetUpError::NotFound(ulid) if ulid == id));
    Ok(())
}

#[::tokio::test]
async fn get_meet_up_metadata_not_found() -> Result<()> {
    let gateway = build_gateway().await?;
    let id = Ulid::new();
    let err = get_meet_up_metadata(&gateway, id)
        .await
        .expect_err("Should error out");
    assert!(matches!(err, GetPastMeetUpError::NotFound(ulid) if ulid == id));
    Ok(())
}

#[::tokio::test]
async fn get_meet_up_metadata_should_return_only_when_done() -> Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let meet_up_call_for_papers = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::CallForPapers,
    )
    .await?;
    let meet_up_voting = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-13".parse()?,
        MeetUpState::Voting,
    )
    .await?;
    let meet_up_scheduled = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-14".parse()?,
        MeetUpState::Scheduled(Paper {
            id: Ulid::new(),
            email: "test@email.com".into(),
            user_id: user.id,
            title: "Some title 1".into(),
            description: "Some description".into(),
            speaker: "Some speaker".into(),
        }),
    )
    .await?;
    let meet_up_done = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-15".parse()?,
        MeetUpState::Done {
            paper: Paper {
                id: Ulid::new(),
                email: "test@email.com".into(),
                user_id: user.id,
                title: "Some title 2".into(),
                description: "Some description".into(),
                speaker: "Some speaker".into(),
            },
            link: "https://example.com".parse()?,
        },
    )
    .await?;
    let err_call_for_papers = get_meet_up_metadata(&gateway, meet_up_call_for_papers.id)
        .await
        .expect_err("Should error out");
    let err_voting = get_meet_up_metadata(&gateway, meet_up_voting.id)
        .await
        .expect_err("Should error out");
    let err_scheduled = get_meet_up_metadata(&gateway, meet_up_scheduled.id)
        .await
        .expect_err("Should error out");
    let meet_up_metadata_done = get_meet_up_metadata(&gateway, meet_up_done.id).await?;
    assert!(
        matches!(err_call_for_papers, GetPastMeetUpError::NotFound(ulid) if ulid == meet_up_call_for_papers.id)
    );
    assert!(matches!(err_voting, GetPastMeetUpError::NotFound(ulid) if ulid == meet_up_voting.id));
    assert!(
        matches!(err_scheduled, GetPastMeetUpError::NotFound(ulid) if ulid == meet_up_scheduled.id)
    );
    assert_eq!("Some title 2", meet_up_metadata_done.title);
    assert_eq!(
        "2024-12-15".parse::<NaiveDate>()?,
        meet_up_metadata_done.date
    );
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_voting_without_future_meet_up() -> Result<()> {
    let gateway = build_gateway().await?;
    let err = move_future_meet_up_to_voting(&gateway)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_voting_with_invalid_meet_up_state() -> Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::Voting,
    )
    .await?;
    let err = move_future_meet_up_to_voting(&gateway)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: Voting", err.to_string());
    assert_meet_up_state!(gateway, meet_up.id, MeetUpState::Voting);
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_voting() -> Result<()> {
    let gateway = build_gateway().await?;
    let mut created_meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::CallForPapers,
    )
    .await?;
    let meet_up = move_future_meet_up_to_voting(&gateway).await?;
    created_meet_up.state = MeetUpState::Voting;
    assert_eq!(created_meet_up, meet_up);
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_scheduled_without_future_meet_up() -> Result<()> {
    let gateway = build_gateway().await?;
    let err = move_future_meet_up_to_scheduled(&gateway, &gateway)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_scheduled_invalid_state() -> Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::CallForPapers,
    )
    .await?;
    let err = move_future_meet_up_to_scheduled(&gateway, &gateway)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: CallForPapers", err.to_string());
    assert_meet_up_state!(gateway, meet_up.id, MeetUpState::CallForPapers);
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_scheduled_without_papers() -> Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::Voting,
    )
    .await?;
    let err = move_future_meet_up_to_scheduled(&gateway, &gateway)
        .await
        .expect_err("Should error out");
    assert_eq!("No valid paper found", err.to_string());
    assert_meet_up_state!(gateway, meet_up.id, MeetUpState::Voting);
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_scheduled() -> Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let mut created_meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::Voting,
    )
    .await?;
    gateway
        .store_paper_with_meet_up(&paper, &created_meet_up.id, 100)
        .await?;
    gateway
        .store_votes(vec![Vote {
            paper_id: paper.id.clone(),
            meet_up_id: created_meet_up.id.clone(),
            user_id: user.id,
            vote: 1.0,
        }])
        .await?;
    let meet_up = move_future_meet_up_to_scheduled(&gateway, &gateway).await?;
    created_meet_up.state = MeetUpState::Scheduled(paper);
    assert_eq!(created_meet_up, meet_up);
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_done_future_meet_up() -> Result<()> {
    let gateway = build_gateway().await?;
    let err = move_future_meet_up_to_done(&gateway, "https://example.com".parse()?)
        .await
        .expect_err("Should error out");
    assert_eq!("No future meetups found", err.to_string());
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_done_invalid_state() -> Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::CallForPapers,
    )
    .await?;
    let err = move_future_meet_up_to_done(&gateway, "https://example.com".parse()?)
        .await
        .expect_err("Should error out");
    assert_eq!("Invalid meet up state: CallForPapers", err.to_string());
    assert_meet_up_state!(gateway, meet_up.id, MeetUpState::CallForPapers);
    Ok(())
}

#[::tokio::test]
async fn move_meet_up_to_done() -> Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let mut meet_up = create_meet_up(
        &gateway,
        "location".to_string(),
        "2024-12-12".parse()?,
        MeetUpState::Scheduled(paper.clone()),
    )
    .await?;
    move_future_meet_up_to_done(&gateway, "https://example.com".parse()?).await?;
    meet_up.state = MeetUpState::Done {
        paper,
        link: "https://example.com".parse()?,
    };
    assert_eq!(meet_up, get_meet_up(&gateway, meet_up.id).await?);
    Ok(())
}
