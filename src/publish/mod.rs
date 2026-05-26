use crate::{catalog::AssetType, Client, RoboatError};

mod request_types;
use bytes::{self, Bytes};
#[derive(Debug, Clone)]
/// An struct to Make a new animation
pub struct AssetToPublish {
    pub name: String,
    pub description: String,
    pub asset_type: AssetType,
    pub asset_data: Bytes,
    pub group_id: Option<u64>,
}

const PUBLISH_API: &str = "https://publish.roblox.com";
impl Client {
    ///a
    pub async fn publish_audio(&self) {}

    ///a
    pub async fn verify_audio(&self) {}
}
mod internal {

    use crate::{
        publish::{
            request_types::PublishAudioRequest, request_types::PublishAudioResponse,
            AssetToPublish, PUBLISH_API,
        },
        Client, RoboatError, XCSRF_HEADER,
    };

    use reqwest::header::{self, USER_AGENT};
    impl Client {
        pub(super) async fn publish_audio_internal(
            &self,
            audio_info: AssetToPublish,
        ) -> Result<PublishAudioResponse, RoboatError> {
            let cookie = &self.cookie_string()?;
            let xcsrf = self.xcsrf().await;

            let payment_source = if audio_info.group_id.is_some() {
                "Group".to_string()
            } else {
                "User".to_string()
            };

            // Convert Bytes to base64 string Return bad request if the bytes are invalid
            let file_base64 = String::from_utf8(audio_info.asset_data)
                .map_err(|_| RoboatError::MalformedResponse)?;

            let request_body = PublishAudioRequest {
                name: audio_info.name,
                base64_file: file_base64,
                group_id: audio_info.group_id,
                payment_source,
            };

            let request_result = self
                .reqwest_client
                .post(format!("{}/v1/audio", PUBLISH_API))
                .header(header::COOKIE, cookie)
                .body(request_body)
                .header(XCSRF_HEADER, xcsrf)
                .header(USER_AGENT, "Roblox/WinInet")
                .send()
                .await;

            if !response.status().is_success() {
                return Err(RoboatError::RequestFailed(response.status()));
            }

            let publish_response: PublishAudioResponse = response.json().await?;
            Ok(publish_response)
        }
    }
}
