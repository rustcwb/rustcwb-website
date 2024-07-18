use domain::{register_event_goer, show_home_page, Location, MeetUpMetadata, MeetUpState};
use shared::utc_now;
use tests::{build_gateway, build_paper_with_user, create_meet_up, create_random_user};

#[::tokio::test]
async fn show_home_page_with_no_entities() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let (meet_up, meet_ups_metadata, is_registered_user) =
        show_home_page(&gateway, &gateway, None).await?;
    assert_eq!(None, meet_up);
    assert_eq!(false, is_registered_user);
    assert_eq!(Vec::<MeetUpMetadata>::new(), meet_ups_metadata);
    Ok(())
}

#[::tokio::test]
async fn show_home_page_with_future_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    let (home_meet_up, meet_ups_metadata, is_registered_user) =
        show_home_page(&gateway, &gateway, None).await?;
    assert_eq!(Some(meet_up), home_meet_up);
    assert_eq!(false, is_registered_user);
    assert_eq!(Vec::<MeetUpMetadata>::new(), meet_ups_metadata);
    Ok(())
}

#[::tokio::test]
async fn show_home_page_with_future_and_past_meet_ups() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let past_meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::Done {
            paper: paper.clone(),
            link: "https://example.com".parse()?,
        },
    )
    .await?;
    let future_meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    let (home_meet_up, meet_ups_metadata, is_registered_user) =
        show_home_page(&gateway, &gateway, Some(&user.id)).await?;
    assert_eq!(Some(future_meet_up), home_meet_up);
    assert_eq!(false, is_registered_user);
    assert_eq!(
        vec![MeetUpMetadata::new(
            past_meet_up.id,
            paper.title,
            past_meet_up.date,
        )],
        meet_ups_metadata
    );
    Ok(())
}

#[::tokio::test]
async fn show_home_page_with_user_registered_to_meet_up_but_not_logged() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let future_meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::Scheduled(paper),
    )
    .await?;
    register_event_goer(&gateway, &gateway, &user.id).await?;
    let (home_meet_up, _, is_registered_user) = show_home_page(&gateway, &gateway, None).await?;
    assert_eq!(Some(future_meet_up), home_meet_up);
    assert_eq!(false, is_registered_user);
    Ok(())
}

#[::tokio::test]
async fn show_home_page_with_user_registered_to_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let future_meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::Scheduled(paper),
    )
    .await?;
    register_event_goer(&gateway, &gateway, &user.id).await?;
    let (home_meet_up, _, is_registered_user) =
        show_home_page(&gateway, &gateway, Some(&user.id)).await?;
    assert_eq!(Some(future_meet_up), home_meet_up);
    assert_eq!(true, is_registered_user);
    Ok(())
}
