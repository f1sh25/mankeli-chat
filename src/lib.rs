pub mod api;
pub mod comms;
pub mod db;

use crate::api::FriendRequestStatus;
pub trait StatusLabel {
    fn status_str(&self) -> &'static str;
    fn status_enum(&self) -> FriendRequestStatus;
}

impl StatusLabel for i64 {
    fn status_enum(&self) -> FriendRequestStatus {
        match *self {
            0 => FriendRequestStatus::InviteSent,
            1 => FriendRequestStatus::InviteReceived,
            2 => FriendRequestStatus::Accepted,
            3 => FriendRequestStatus::Rejected,
            _ => FriendRequestStatus::Rejected,
        }
    }

    fn status_str(&self) -> &'static str {
        match *self {
            0 => "invite_sent",
            1 => "invite_received",
            2 => "accepted",
            3 => "rejected",
            _ => "unknown",
        }
    }
}
