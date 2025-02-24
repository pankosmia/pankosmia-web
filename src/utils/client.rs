use std::sync::Mutex;
use crate::structs::{Client, PublicClient};

pub(crate) fn public_serialize_client(c: Client) -> PublicClient {
    PublicClient {
        id: c.id.clone(),
        requires: c.requires.clone(),
        exclude_from_menu: c.exclude_from_menu.clone(),
        exclude_from_dashboard: c.exclude_from_dashboard.clone(),
        url: c.url.clone(),
    }
}
pub(crate) fn public_serialize_clients(cv: Vec<Client>) -> Vec<PublicClient> {
    cv.into_iter().map(|c| public_serialize_client(c)).collect()
}
pub(crate) type Clients = Mutex<Vec<Client>>;
