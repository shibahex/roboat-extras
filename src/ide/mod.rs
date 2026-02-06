use crate::{ide::ide_types::NewStudioAsset, Client, RoboatError};

/// Types for all the IDE API
pub mod ide_types;

const STUDIO_UPLOAD_API: &str = "https://www.roblox.com/ide/publish/uploadnewanimation";

// IDE is used for private APIs like ide/uploadnewanimation and ide/places/createV2

impl Client {
    /// Uploads a new (animation, audio or decal) asset to Roblox using the internal `ide/publish/uploadnewanimation` endpoint.
    ///
    /// # Endpoint
    /// Sends a `POST` request to `https://www.roblox.com/ide/publish/uploadnewanimation`
    /// with animation/audio/image metadata as query parameters and animation binary data in the body.
    ///
    /// # Notes
    /// * Requires a valid `.ROBLOSECURITY` cookie for authentication.
    /// * (`asset_data`) must be provided as binary (e.g., R15 animation XML).
    /// * If the X-CSRF token is expired or invalid, it will retry the request once with a refreshed token.
    ///
    /// # Query Parameters Changeable
    /// Automatically included in the request URL:
    /// * `name` – The title of the asset
    /// * `description` – The description of the asset
    ///
    /// # Optional Params
    /// * `groupId` – Optional group ID (if uploading to a group)
    ///
    /// # Return Value Notes
    /// * Returns `String` of the new asset ID if the asset was uploaded successfully.
    /// * Or Returns an error.
    ///
    /// # Errors
    /// * [RoboatError::MissingAuth] – If the `.ROBLOSECURITY` cookie is missing.
    /// * [RoboatError::InvalidXcsrf] – If the CSRF token needs refreshing (retry will be attempted).
    /// * [RoboatError::ReqwestError] – For any network issues.
    /// * [RoboatError::ResponseError] – If Roblox returns a failure response.
    ///
    /// # Example
    /// ```no_run
    /// use bytes::Bytes;
    /// use roboat::{ClientBuilder, ide::request_types::Animation};
    ///
    /// const ROBLOSECURITY: &str = "your_.ROBLOSECURITY_cookie";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .roblosecurity(ROBLOSECURITY.to_string())
    ///     .build();
    ///
    /// let animation = NewStudioAsset {
    ///     title: "MyCoolAnimation".to_string(),
    ///     description: "A test animation created by Roboat.".to_string(),
    ///     group_id: Some(123456),
    ///     animation_data: Some(Bytes::from_static(b"<KeyframeSequence>...</KeyframeSequence>")),
    /// };
    ///
    /// client.upload_studio_asset(animation).await?;
    ///
    /// println!("Successfully uploaded animation.");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_studio_asset(
        &self,
        asset_info: NewStudioAsset,
    ) -> Result<String, RoboatError> {
        match self.upload_studio_asset_internal(asset_info.clone()).await {
            Ok(x) => Ok(x),
            Err(RoboatError::InvalidXcsrf(new_xcsrf)) => {
                self.set_xcsrf(new_xcsrf).await;
                self.upload_studio_asset_internal(asset_info).await
            }
            Err(e) => Err(e),
        }
    }
}

mod internal {
    use crate::{
        ide::{ide_types::NewStudioAsset, STUDIO_UPLOAD_API},
        Client, RoboatError, XCSRF_HEADER,
    };
    use reqwest::header::{self, USER_AGENT};
    impl Client {
        pub(super) async fn upload_studio_asset_internal(
            &self,
            asset_info: NewStudioAsset,
        ) -> Result<String, RoboatError> {
            let cookie = self.cookie_string()?;
            let xcsrf = self.xcsrf().await;

            // asset_info.asset_type.into(str)
            let mut query_params = vec![
                ("assetTypeName", format!("{:?}", asset_info.asset_type)),
                ("name", asset_info.name.clone()),
                ("description", asset_info.description.clone()),
                ("AllID", "1".to_string()),
                ("ispublic", "False".to_string()),
                ("allowComments", "True".to_string()),
                ("isGamesAsset", "False".to_string()),
            ];

            // Add groupID if exists
            if let Some(group_id) = asset_info.group_id {
                query_params.push(("groupId", group_id.to_string()));
            }

            let request_result = self
                .reqwest_client
                .post(STUDIO_UPLOAD_API)
                .query(&query_params)
                .header(header::COOKIE, cookie)
                .body(asset_info.asset_data)
                .header(XCSRF_HEADER, xcsrf)
                .header(USER_AGENT, "Roblox/WinInet")
                .send()
                .await;

            let response = Self::validate_request_result(request_result).await?;
            let response_id = response.text().await.map_err(RoboatError::ReqwestError)?;
            Ok(response_id)
        }
    }
}
