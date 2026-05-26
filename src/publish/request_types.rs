use bytes::Bytes;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PublishAudioRequest {
    pub name: String,
    pub file_byes: String,
    pub group_id: Option<u64>,
    pub payment_source: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub(super) struct PublishAudioResponse {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct VerifyAudioResponse {
    pub price: u64,
    pub can_afford: bool,
}
