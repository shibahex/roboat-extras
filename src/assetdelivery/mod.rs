// mod request_types;
#![allow(missing_docs)]

const ASSETDELIVERY_ASSET_API: &str = "https://assetdelivery.roblox.com/v1/asset/?ID={id}";
const ASSETDELIVERY_V2_API: &str = "https://assetdelivery.roblox.com/v2";

/// Open Cloud version of the asset delivery API. Requires an API key via `x-api-key` header.
const OPEN_CLOUD_ASSET_DELIVERY_API: &str = "https://apis.roblox.com/asset-delivery-api/v1/assetId";

use crate::catalog::AssetType;
use crate::validation::RobloxErrorRaw;
use crate::{Client, RoboatError, XCSRF_HEADER};
use bytes::Bytes;
use reqwest::header;
use serde_with::skip_serializing_none;

/// All the payload/response structs
pub mod request_types;

use serde::Deserialize;
use serde::Serialize;

//https://create.roblox.com/docs/cloud/legacy/assetdelivery/v2#/AssetFetchV2/get_v2_assetId__assetId_
/// Structs For MetaData /v2/assetId/{assetId} Endpoint
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetIdResponse {
    pub errors: Option<Vec<RobloxErrorRaw>>,
    pub locations: Vec<request_types::AssetLocation>,
    pub request_id: String,
    #[serde(rename = "IsHashDynamic")]
    pub is_hash_dynamic: bool,
    #[serde(rename = "IsCopyrightProtected")]
    pub is_copyright_protected: bool,
    pub is_archived: bool,
    pub asset_type_id: u64,
    pub is_recordable: bool,
}

/// Struct for the Open Cloud <https://apis.roblox.com/asset-delivery-api/v1/assetId/{assetId}> endpoint.
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CloudAssetLocation {
    /// A temporary CDN url for the asset content.
    pub location: String,
}

/// Structs for /v2/assets/batch All of these are optional to define what Asset you want information from.
//{https://create.roblox.com/docs/cloud/legacy/assetdelivery/v2#/BatchV2/post_v2_assets_batch}
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(default)]
#[serde(rename_all = "camelCase")]
pub struct AssetBatchPayload {
    pub asset_name: Option<String>,
    pub asset_type: Option<String>,
    pub client_insert: Option<bool>,
    pub place_id: Option<String>,
    pub request_id: Option<String>,
    pub script_insert: Option<bool>,
    pub server_place_id: Option<String>,
    pub universe_id: Option<String>,
    pub accept: Option<String>,
    pub encoding: Option<String>,
    pub hash: Option<String>,
    pub user_asset_id: Option<String>,
    pub asset_id: Option<String>,
    pub version: Option<String>,
    pub asset_version_id: Option<String>,
    pub module_place_id: Option<String>,
    pub asset_format: Option<String>,
    #[serde(rename = "roblox-assetFormat")]
    pub roblox_asset_format: Option<String>,
    pub content_representation_priority_list: Option<String>,
    pub do_not_fallback_to_baseline_representation: Option<bool>,
}

/// Response for AssetBatch
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetBatchResponse {
    pub errors: Option<Vec<RobloxErrorRaw>>,
    pub locations: Option<Vec<request_types::Location>>,
    pub request_id: Option<String>,
    pub is_hash_dynamic: Option<bool>,
    pub is_copyright_protected: Option<bool>,
    pub is_archived: Option<bool>,
    pub asset_type_id: Option<u8>,
    pub asset_type: Option<AssetType>,
    pub content_representation_specifier: Option<request_types::ContentRepresentationSpecifier>,
    pub is_recordable: Option<bool>,
}
/// WARNING: Some AssetDelivery V2 API returns Errors even when its status code 200
impl Client {
    /// Gets Meta data from item, this works on Animations, outfits, places and other assets.
    /// This will return <AssetMetaData> which has asset_type_id and the download for the file.
    /// Also more information like if the item is deleted.
    ///
    /// # Notes
    /// Requires valid roblosecurity
    ///
    /// Add [`Roblox-Place-Id`] in the request headers if the asset is not owned by your account.
    /// The place-id can be any place the owner of the asset has
    ///
    /// *Can return a sucess but still have error codes in the response
    /// Doesn't need xcrf, but will add one if it gets 401
    pub async fn fetch_asset_metadata(
        &self,
        asset_id: u64,
    ) -> Result<AssetIdResponse, RoboatError> {
        match self.fetch_asset_metadata_internal(asset_id).await {
            Ok(x) => Ok(x),
            Err(e) => match e {
                RoboatError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;
                    self.fetch_asset_metadata_internal(asset_id).await
                }
                _ => Err(e),
            },
        }
    }

    /// Sends a batch request to fetch metadata for multiple assets.
    ///
    /// This method accepts a vector of `AssetBatchPayload` items, sends them
    /// to the Roblox AssetDelivery v2 batch API endpoint, and returns a vector
    /// of `AssetBatchResponse` objects corresponding to each requested asset.
    ///
    /// # Parameters
    /// - `asset_batch`: A vector of `AssetBatchPayload` structs representing
    ///   the assets for which metadata is requested.
    ///
    /// # Notes
    /// Needs Roblox Cookie but not CSRF
    ///
    /// Add [`Roblox-Place-Id`] in the request headers if the asset is not owned by your account.
    /// The place-id can be any place the owner of the asset has
    ///
    /// Can return a sucess but still have error codes in the response
    ///     
    /// # Returns
    /// Returns a `Result` containing a vector of `AssetBatchResponse` on success,
    /// or a `RoboatError` if the request fails or the response is malformed.
    ///
    /// # Behavior
    /// - Automatically handles `InvalidXcsrf` errors by refreshing the X-CSRF token
    ///   and retrying the request once.
    /// - Requires a valid `.ROBLOSECURITY` cookie set in the `Client`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use clap::Parser;
    /// use roboat::assetdelivery::AssetBatchPayload;
    ///
    /// #[derive(Parser, Debug)]
    /// struct Args {
    ///     #[arg(long, short)]
    ///     roblosecurity: String,
    /// }
    ///
    /// use roboat::ClientBuilder;
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let args = Args::parse();
    ///     let client = ClientBuilder::new()
    ///         .roblosecurity(args.roblosecurity)
    ///         .build();
    ///     let payloads = vec![AssetBatchPayload {
    ///         asset_id: Some("105277031944789".to_string()),
    ///         request_id: Some("0".to_string()),
    ///         ..Default::default()
    ///     }];
    ///     let responses = client.post_asset_metadata_batch(payloads).await?;
    ///     for response in responses {
    ///         println!("Response for request {:?}", response);
    ///     }
    ///     Ok(())
    ///}
    /// ```
    ///
    /// # Errors
    /// This function will return errors including but not limited to:
    /// - Network or HTTP errors from the reqwest client
    /// - Invalid or expired X-CSRF tokens (which it will attempt to refresh automatically)
    /// - Malformed responses from the API
    pub async fn post_asset_metadata_batch(
        &self,
        asset_batch: Vec<AssetBatchPayload>,
    ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
        match self
            .post_asset_metadata_batch_internal(asset_batch.clone())
            .await
        {
            Ok(x) => Ok(x),
            Err(e) => match e {
                RoboatError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;
                    self.post_asset_metadata_batch_internal(asset_batch).await
                }
                _ => Err(e),
            },
        }
    }

    /// Downloads a raw asset bytes using the endpoint <https://assetdelivery.roblox.com/v1/asset/?id={id}>.
    ///
    /// # Notes
    /// * Requires Gzip feature on reqwest for automatic decompression of the data.
    /// * Requires a valid `.ROBLOSECURITY` cookie for private or restricted assets.
    /// * Returns the raw binary content of the asset (e.g., `.rbxm`, `.rbxmx`, etc.).
    /// * The use case for this API, would be download Animation, Models and outfits.
    /// * very usefully for studio projects
    ///
    /// # Errors
    /// * [Standard Errors](#standard-errors)
    /// * [Auth Required Errors](#auth-required-errors)
    ///
    /// # Example
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "your_cookie";
    /// const ASSET_ID: u64 = 12345678;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .roblosecurity(ROBLOSECURITY.to_string())
    ///     .build();
    ///
    /// let data = client.fetch_asset_data(ASSET_ID).await?;
    /// println!("Downloaded {} bytes", data.len());
    /// # Ok(())
    /// # }
    /// ```
    // NOTE: Roblox discourages using a cookie on assetdelivery.roblox.com. Prefer
    // Client::fetch_cloud_asset_data, which uses an API key.
    /// If this API hangs, use a timeout and retry.
    pub async fn fetch_asset_data(&self, asset_id: u64) -> Result<Bytes, RoboatError> {
        let cookie_string = self.cookie_string()?;
        let formatted_url = ASSETDELIVERY_ASSET_API.replace("{id}", &asset_id.to_string());

        let xcsrf = self.xcsrf().await;

        let response_result = self
            .reqwest_client
            .get(&formatted_url)
            .header(header::COOKIE, cookie_string)
            .header(XCSRF_HEADER, xcsrf)
            .send()
            .await;

        let response = Self::validate_request_result(response_result).await?;

        let bytes = response.bytes().await.map_err(RoboatError::ReqwestError)?;
        Ok(bytes)
    }

    /// Gets a temporary download url for an asset using the Open Cloud endpoint
    /// <https://apis.roblox.com/asset-delivery-api/v1/assetId/{assetId}>.
    ///
    /// This is the Open Cloud version of [`Client::fetch_asset_metadata`], and works on all asset types.
    ///
    /// # Notes
    /// * Requires a valid API key with the `legacy-asset:manage` scope. A roblosecurity is not needed.
    /// * Only works with API keys made by a user, not a group.
    /// * Add `place_id` if the asset is not owned by the API key owner.
    ///   The place id can be any place the owner of the asset has.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * [`RoboatError::InvalidRoblosecurity`] - If the API key is invalid or is missing the scope.
    /// * [`RoboatError::MalformedResponse`] - If the response from the server cannot be parsed.
    ///
    /// # Example
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const API_KEY: &str = "your_api_key";
    /// const ASSET_ID: u64 = 12345678;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().build();
    ///
    /// let asset = client
    ///     .fetch_cloud_asset_location(API_KEY.to_string(), ASSET_ID, None)
    ///     .await?;
    /// println!("Download url: {}", asset.location);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_cloud_asset_location(
        &self,
        api_key: String,
        asset_id: u64,
        place_id: Option<u64>,
    ) -> Result<CloudAssetLocation, RoboatError> {
        self.fetch_cloud_asset_location_internal(api_key, asset_id, place_id)
            .await
    }

    /// Downloads the raw bytes of an asset using the Open Cloud endpoint
    /// <https://apis.roblox.com/asset-delivery-api/v1/assetId/{assetId}>.
    ///
    /// This is the Open Cloud version of [`Client::fetch_asset_data`]. It gets the download url
    /// with [`Client::fetch_cloud_asset_location`], then downloads the file from it.
    ///
    /// # Notes
    /// * Requires a valid API key with the `legacy-asset:manage` scope. A roblosecurity is not needed.
    /// * Only works with API keys made by a user, not a group.
    /// * Add `place_id` if the asset is not owned by the API key owner.
    ///   The place id can be any place the owner of the asset has.
    /// * Requires Gzip feature on reqwest for automatic decompression of the data.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * [`RoboatError::InvalidRoblosecurity`] - If the API key is invalid or is missing the scope.
    /// * [`RoboatError::MalformedResponse`] - If the response from the server cannot be parsed.
    ///
    /// # Example
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const API_KEY: &str = "your_api_key";
    /// const ASSET_ID: u64 = 12345678;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().build();
    ///
    /// let data = client
    ///     .fetch_cloud_asset_data(API_KEY.to_string(), ASSET_ID, None)
    ///     .await?;
    /// println!("Downloaded {} bytes", data.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn fetch_cloud_asset_data(
        &self,
        api_key: String,
        asset_id: u64,
        place_id: Option<u64>,
    ) -> Result<Bytes, RoboatError> {
        let asset = self
            .fetch_cloud_asset_location(api_key, asset_id, place_id)
            .await?;

        let response_result = self.reqwest_client.get(&asset.location).send().await;
        let response = Self::validate_request_result(response_result).await?;

        let bytes = response.bytes().await.map_err(RoboatError::ReqwestError)?;
        Ok(bytes)
    }
}

mod internal {
    use crate::assetdelivery::AssetBatchPayload;
    use crate::assetdelivery::AssetBatchResponse;
    use crate::assetdelivery::AssetIdResponse;
    use crate::assetdelivery::CloudAssetLocation;
    use crate::assetdelivery::ASSETDELIVERY_V2_API;
    use crate::assetdelivery::OPEN_CLOUD_ASSET_DELIVERY_API;
    use crate::catalog::catalog_types;
    use crate::{Client, RoboatError};
    use reqwest::header;

    impl Client {
        pub(super) async fn post_asset_metadata_batch_internal(
            &self,
            asset_payload: Vec<AssetBatchPayload>,
        ) -> Result<Vec<AssetBatchResponse>, RoboatError> {
            let cookie = self.cookie_string()?;
            let formatted_url = format!("{ASSETDELIVERY_V2_API}/assets/batch");

            let request_result = self
                .reqwest_client
                .post(formatted_url)
                .header(header::COOKIE, cookie)
                .json(&asset_payload)
                .send()
                .await;

            let response = Self::validate_request_result(request_result).await?;
            let mut meta_data = Self::parse_to_raw::<Vec<AssetBatchResponse>>(response).await?;

            // Scan response for roblox errors, if its 401 just return Invalid Cookie (Can't be
            // CSRF on this API)
            for batch_resp in &mut meta_data {
                if let Some(id) = batch_resp.asset_type_id {
                    if let Ok(e) = catalog_types::AssetType::try_from(id as u64) {
                        batch_resp.asset_type = Some(e);
                    }
                }
                if let Some(roblox_error_raw) = &batch_resp.errors {
                    for error in roblox_error_raw {
                        // 401 Error will be .ROBLOSECURITY. and not CSRF.
                        if error.code == 401 {
                            return Err(RoboatError::InvalidRoblosecurity);
                        }
                    }
                }
            }

            Ok(meta_data)
        }
        pub(super) async fn fetch_asset_metadata_internal(
            &self,
            asset_id: u64,
        ) -> Result<AssetIdResponse, RoboatError> {
            let cookie = self.cookie_string()?;
            // let xcsrf = self.xcsrf().await;
            let formatted_url = format!("{}/assetid/{}", ASSETDELIVERY_V2_API, asset_id);

            let request_result = self
                .reqwest_client
                .get(formatted_url)
                .header(header::COOKIE, cookie)
                .send()
                .await;

            let response = Self::validate_request_result(request_result).await?;
            let meta_data = Self::parse_to_raw::<AssetIdResponse>(response).await?;

            // Scan response for roblox errors, if its 401 just return Invalid Cookie (Can't be
            // CSRF on this API)
            if let Some(roblox_error_raw) = &meta_data.errors {
                // We can only return one error, so we just return the first
                let first_error = roblox_error_raw.first().unwrap();
                // 401 Error will be .ROBLOSECURITY. and not CSRF.
                if first_error.code == 401 {
                    return Err(RoboatError::InvalidRoblosecurity);
                } else {
                    // this API either gets one Error or Asset. Return the Err
                    // NOTE: This Error could be that the Asset is Private.
                    return Err(RoboatError::UnidentifiedStatusCode(first_error.code));
                }
            }

            Ok(meta_data)
        }

        pub(super) async fn fetch_cloud_asset_location_internal(
            &self,
            api_key: String,
            asset_id: u64,
            place_id: Option<u64>,
        ) -> Result<CloudAssetLocation, RoboatError> {
            let formatted_url = format!("{}/{}", OPEN_CLOUD_ASSET_DELIVERY_API, asset_id);

            let mut request = self
                .reqwest_client
                .get(formatted_url)
                .header("x-api-key", api_key);

            if let Some(place_id) = place_id {
                request = request.header("Roblox-Place-Id", place_id.to_string());
            }

            let response = Self::validate_request_result(request.send().await).await?;
            let location = Self::parse_to_raw::<CloudAssetLocation>(response).await?;

            Ok(location)
        }
    }
}
