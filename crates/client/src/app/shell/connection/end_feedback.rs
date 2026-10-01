use leocard_client::{NetworkState, TcpGameClient};
pub(crate) struct ConnectionEndFeedback<'a> {
    pub client: &'a TcpGameClient,
    pub was_host: bool,
    pub leaving_room: bool,
}
impl ConnectionEndFeedback<'_> {
    pub(crate) fn message(&self) -> Option<Option<String>> {
        if self.client.model().room_closed() {
            Some((!self.was_host && !self.leaving_room).then(|| "房主结束了游戏".to_owned()))
        } else if self.client.model().left_room() {
            Some(None)
        } else if let NetworkState::Failed(error) = self.client.state() {
            Some((!self.leaving_room).then(|| error.clone()))
        } else {
            None
        }
    }
}
