use domain::{
    register_event_goer, show_admin_page, submit_paper, Location, MeetUpState,
    ShowAdminPageResponse,
};
use shared::utc_now;
use tests::{build_gateway, build_paper_with_user, create_meet_up, create_random_user};

#[::tokio::test]
async fn show_admin_page_without_future_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let response = show_admin_page(&gateway, &gateway, &gateway).await?;
    assert_eq!(ShowAdminPageResponse::NoMeetUp, response);
    Ok(())
}

#[::tokio::test]
async fn show_admin_page_with_future_meet_up_and_no_papers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    let response = show_admin_page(&gateway, &gateway, &gateway).await?;
    assert_eq!(
        ShowAdminPageResponse::MeetUpWithPapers(meet_up, 0),
        response
    );
    Ok(())
}

#[::tokio::test]
async fn show_admin_page_with_future_meet_up_and_papers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let user = create_random_user(&gateway).await?;
    let paper = build_paper_with_user(user.id.clone());
    let meet_up = create_meet_up(
        &gateway,
        Location::OnSite("location".into()),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
    .await?;
    submit_paper(&gateway, &gateway, paper).await?;

    let response = show_admin_page(&gateway, &gateway, &gateway).await?;
    assert_eq!(
        ShowAdminPageResponse::MeetUpWithPapers(meet_up, 1),
        response
    );
    Ok(())
}

#[::tokio::test]
async fn show_admin_page_with_attendees() -> anyhow::Result<()> {
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
    register_event_goer(&gateway, &gateway, &user.id).await?;

    let response = show_admin_page(&gateway, &gateway, &gateway).await?;
    assert_eq!(
        ShowAdminPageResponse::MeetUpWithAttendees(meet_up, 1),
        response
    );
    Ok(())
}
