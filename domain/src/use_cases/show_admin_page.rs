use crate::{MeetUp, MeetUpGateway, MeetUpGoersGateway, MeetUpState, PaperGateway};

pub async fn show_admin_page(
    meet_up_gateway: &impl MeetUpGateway,
    papers_gateway: &impl PaperGateway,
    meet_up_goers_gateway: &impl MeetUpGoersGateway,
) -> anyhow::Result<ShowAdminPageResponse> {
    let future_meet_up = meet_up_gateway.get_future_meet_up().await?;
    Ok(match future_meet_up {
        None => ShowAdminPageResponse::NoMeetUp,
        Some(meet_up) => match &meet_up.state {
            MeetUpState::CallForPapers | MeetUpState::Voting => {
                let n_papers = papers_gateway
                    .get_papers_from_meet_up(&meet_up.id)
                    .await?
                    .len();
                ShowAdminPageResponse::MeetUpWithPapers(meet_up, n_papers)
            }
            MeetUpState::Scheduled(..) => {
                let n_attendees = meet_up_goers_gateway
                    .get_number_attendees_from_meet_up(&meet_up.id)
                    .await?;
                ShowAdminPageResponse::MeetUpWithAttendees(meet_up, n_attendees)
            }
            _ => ShowAdminPageResponse::MeetUp(meet_up),
        },
    })
}

#[derive(Debug, PartialEq, Eq)]
pub enum ShowAdminPageResponse {
    NoMeetUp,
    MeetUp(MeetUp),
    MeetUpWithPapers(MeetUp, usize),
    MeetUpWithAttendees(MeetUp, usize),
}

impl ShowAdminPageResponse {
    pub fn n_papers(&self) -> Option<usize> {
        match self {
            ShowAdminPageResponse::MeetUpWithPapers(_, n_papers) => Some(*n_papers),
            _ => None,
        }
    }

    pub fn n_attendees(&self) -> Option<usize> {
        match self {
            ShowAdminPageResponse::MeetUpWithAttendees(_, n_attendees) => Some(*n_attendees),
            _ => None,
        }
    }

    pub fn into_meet_up(self) -> Option<MeetUp> {
        match self {
            ShowAdminPageResponse::MeetUp(meet_up)
            | ShowAdminPageResponse::MeetUpWithPapers(meet_up, _)
            | ShowAdminPageResponse::MeetUpWithAttendees(meet_up, _) => Some(meet_up),
            ShowAdminPageResponse::NoMeetUp => None,
        }
    }
}
