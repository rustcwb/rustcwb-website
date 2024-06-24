use crate::{FutureMeetUp, FutureMeetUpGateway, PaperGateway};

pub async fn show_admin_page(
    future_meet_up_gateway: &impl FutureMeetUpGateway,
    papers_gateway: &impl PaperGateway,
) -> anyhow::Result<(Option<FutureMeetUp>, usize)> {
    let future_meet_up = future_meet_up_gateway.get_future_meet_up().await?;
    let n_papers = match &future_meet_up {
        None => 0,
        Some(future_meet_up) => papers_gateway
            .get_papers_from_meet_up(&future_meet_up.id)
            .await?
            .len(),
    };
    Ok((future_meet_up, n_papers))
}
