use domain::{MeetUpState, show_admin_page, submit_paper};
use shared::utc_now;
use tests::{build_gateway, build_paper_with_user, create_meet_up, create_random_user};

#[::tokio::test]
async fn show_admin_page_without_future_meet_up() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let (meet_up, n_papers) = show_admin_page(&gateway, &gateway).await?;
    assert_eq!(None, meet_up);
    assert_eq!(0, n_papers);
    Ok(())
}

#[::tokio::test]
async fn show_admin_page_with_future_meet_up_and_no_papers() -> anyhow::Result<()> {
    let gateway = build_gateway().await?;
    let meet_up = create_meet_up(
        &gateway,
        "location".into(),
        utc_now().naive_utc().date(),
        MeetUpState::CallForPapers,
    )
        .await?;
    let (admin_meet_up, n_papers) = show_admin_page(&gateway, &gateway).await?;
    assert_eq!(Some(meet_up), admin_meet_up);
    assert_eq!(0, n_papers);
    Ok(())
}

#[::tokio::test]
async fn show_admin_page_with_future_meet_up_and_papers() -> anyhow::Result<()> {
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
    submit_paper(&gateway, &gateway, paper).await?;

    let (admin_meet_up, n_papers) = show_admin_page(&gateway, &gateway).await?;
    assert_eq!(Some(meet_up), admin_meet_up);
    assert_eq!(1, n_papers);
    Ok(())
}
