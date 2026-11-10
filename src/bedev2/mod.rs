use crate::bedev2::request_types::AssetInfo;
use crate::catalog::{AssetType, CreatorType};
use crate::{Client, RoboatError};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
/// All the response structs
pub mod request_types;

const COLLECTIBLE_ITEM_DETAILS_API: &str =
    "https://apis.roblox.com/marketplace-items/v1/items/details";

const PURCHASE_NON_TRADEABLE_LIMITED_API_PART_1: &str =
    "https://apis.roblox.com/marketplace-sales/v1/item/";

/// This API endpoint supports two operations:
/// - POST request with a payload to upload an asset
/// - GET request with an asset ID parameter (/{Id}) to retrieve asset information
const ASSET_API: &str = "https://apis.roblox.com/assets/user-auth/v1/assets";

/// Open Cloud asset upload endpoint. Requires an API key via `x-api-key` header.
/// Supports Animation, Audio, Decal, Model and Video uploads, unlike the user-auth endpoint.
const OPEN_CLOUD_ASSET_API: &str = "https://apis.roblox.com/assets/v1/assets";

/// Open Cloud endpoint for checking an upload. Requires an API key via `x-api-key` header.
const OPEN_CLOUD_OPERATION_API: &str = "https://apis.roblox.com/assets/v1/operations";

const PURCHASE_NON_TRADEABLE_LIMITED_API_PART_2: &str = "/purchase-item";

/// Custom Roblox errors that occur when using [`Client::purchase_non_tradable_limited`].
#[non_exhaustive]
#[derive(
    thiserror::Error,
    Debug,
    Default,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
)]
pub enum PurchaseNonTradableLimitedError {
    /// Thrown when the price of the item is not the same as the price you're trying to buy it for.
    #[default]
    #[error("Price Mismatch")]
    PriceMismatch,
    /// Thrown when the item is sold out.
    #[error("Sold Out")]
    SoldOut,
    /// Thrown when an unknown error occurs.
    #[error("Unknown Roblox Error Message: {0}")]
    UnknownRobloxErrorMsg(String),
}

/// Used to specify the type of classic clothing being uploaded.
///
/// This is used in [`Client::upload_classic_clothing_to_group`].
#[allow(missing_docs)]
#[derive(
    Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize, Copy,
)]
pub enum ClassicClothingType {
    #[default]
    Shirt,
    Pants,
    TShirt,
}

impl std::fmt::Display for ClassicClothingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClassicClothingType::Shirt => write!(f, "Shirt"),
            ClassicClothingType::Pants => write!(f, "Pants"),
            ClassicClothingType::TShirt => write!(f, "T-Shirt"),
        }
    }
}

/// Who an asset is uploaded under, used in [`Client::upload_asset`] and [`Client::upload_animation`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Copy)]
pub enum AssetCreator {
    /// Upload under a user, using their user id.
    User(u64),
    /// Upload under a group, using its group id.
    Group(u64),
}

/// A struct containing (mostly) all the fields possibly returned from <https://apis.roblox.com/marketplace-items/v1/items/details>.
///
/// Returned from [`Client::non_tradable_limited_details`].
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct NonTradableLimitedDetails {
    /// The normal u64 item id.
    pub item_id: u64,
    /// The string item id. This only exists for non-tradable limiteds.
    pub collectible_item_id: String,
    /// The string used as a product id to buy the item.
    pub collectible_product_id: String,
    /// The name of the item.
    pub name: String,
    /// The description of the item.
    pub description: String,
    /// Whether the creator has a verified badge.
    pub creator_has_verified_badge: bool,
    /// The type of creator that created the item (User or Group).
    pub creator_type: CreatorType,
    /// The id (group or user) of the creator. The value is 1 if the creator is Roblox.
    pub creator_id: u64,
    /// The name of the creator. The value is "Roblox" if the creator is Roblox.
    pub creator_name: String,
    /// It's unclear which one of these to use as `price` and `lowest_price`
    /// have been the same every time I have tested this.
    pub price: u64,
    /// It's unclear which one of these to use as `price` and `lowest_price`
    /// have been the same every time I have tested this.
    pub lowest_price: u64,
    /// The stock amount left.
    pub remaining_stock: u64,
    /// The total stock amount on release.
    pub total_stock: u64,
    /// The error code if Roblox decides to throw one.
    pub error_code: Option<u64>,
}

impl TryFrom<request_types::NonTradableLimitedDetailsRaw> for NonTradableLimitedDetails {
    type Error = RoboatError;

    fn try_from(raw: request_types::NonTradableLimitedDetailsRaw) -> Result<Self, Self::Error> {
        let item_id = raw.item_target_id;
        let collectible_item_id = raw.collectible_item_id;
        let collectible_product_id = raw.collectible_product_id;
        let name = raw.name;
        let description = raw.description;
        let creator_has_verified_badge = raw.creator_has_verified_badge;
        let creator_type = raw.creator_type;
        let creator_id = raw.creator_id;
        let creator_name = raw.creator_name;
        let price = raw.price;
        let lowest_price = raw.lowest_price;
        let remaining_stock = raw.units_available_for_consumption;
        let total_stock = raw.asset_stock;
        let error_code = raw.error_code;

        Ok(Self {
            item_id,
            collectible_item_id,
            collectible_product_id,
            name,
            description,
            creator_has_verified_badge,
            creator_type,
            creator_id,
            creator_name,
            price,
            lowest_price,
            remaining_stock,
            total_stock,
            error_code,
        })
    }
}

impl Client {
    /// Grabs details of one or more non-tradable limiteds from <https://apis.roblox.com/marketplace-items/v1/items/details>.
    /// Does not work on normal items. Note that this is a messy,
    /// all-encompassing endpoint that should only be used directly when necessary.
    ///
    /// Specialized endpoints that use this internally include: [`Client::collectible_product_id`] and [`Client::collectible_product_id_bulk`].
    ///
    /// # Notes
    /// * Requires a valid roblosecurity.
    /// * The amount of items that can be requested at once is unknown as not enough non-tradable limiteds exist, and the
    ///   endpoint doesn't accept duplicates.
    /// * Will repeat once if the x-csrf-token is invalid.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [X-CSRF-TOKEN Required Errors](#x-csrf-token-required-errors).
    /// * All errors under [Auth Required Errors](#auth-required-errors).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .roblosecurity(ROBLOSECURITY.to_string())
    ///     .build();
    ///
    /// let collectible_item_id_1 = "a4b5cb79-5218-4ca1-93fa-1e3436f595ef".to_owned();
    /// let collectible_item_id_2 = "61f2e366-9fe6-4562-8ce3-47334083372a".to_owned();
    /// let items = vec![collectible_item_id_1, collectible_item_id_2];
    ///
    /// let details = client.non_tradable_limited_details(items).await?;
    ///
    /// println!("Item Name: {}", details[0].name);
    /// println!("Item Description: {}", details[0].description);
    /// println!("Item Price: {}", details[0].price);
    ///
    /// println!("Item Name: {}", details[1].name);
    /// println!("Item Description: {}", details[1].description);
    /// println!("Item Price: {}", details[1].price);
    ///
    /// # Ok(())
    /// # }
    /// ```
    pub async fn non_tradable_limited_details(
        &self,
        collectible_item_ids: Vec<String>,
    ) -> Result<Vec<NonTradableLimitedDetails>, RoboatError> {
        match self
            .non_tradable_limited_details_internal(collectible_item_ids.clone())
            .await
        {
            Ok(x) => Ok(x),
            Err(e) => match e {
                RoboatError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;

                    self.non_tradable_limited_details_internal(collectible_item_ids)
                        .await
                }
                _ => Err(e),
            },
        }
    }

    /// Fetches the collectible product id of a non-tradeable limited. Uses [`Client::non_tradable_limited_details`] internally
    /// (which fetches from <https://apis.roblox.com/marketplace-items/v1/items/details>)
    ///
    /// # Notes
    /// * Requires a valid roblosecurity.
    /// * Will repeat once if the x-csrf-token is invalid.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [X-CSRF-TOKEN Required Errors](#x-csrf-token-required-errors).
    /// * All errors under [Auth Required Errors](#auth-required-errors).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .roblosecurity(ROBLOSECURITY.to_string())
    ///     .build();
    ///
    /// let collectible_item_id = "a4b5cb79-5218-4ca1-93fa-1e3436f595ef".to_owned();
    /// let collectible_product_id = client.collectible_product_id(collectible_item_id).await?;
    ///
    /// println!("Collectible Product ID: {}", collectible_product_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collectible_product_id(
        &self,
        collectible_item_id: String,
    ) -> Result<String, RoboatError> {
        let details = self
            .non_tradable_limited_details(vec![collectible_item_id])
            .await?;

        let collectible_product_id = details
            .first()
            .ok_or(RoboatError::MalformedResponse)?
            .collectible_product_id
            .clone();

        Ok(collectible_product_id)
    }

    /// Fetches collectible product ids of multiple non-tradeable limiteds. Uses [`Client::non_tradable_limited_details`] internally
    /// (which fetches from <https://apis.roblox.com/marketplace-items/v1/items/details>)
    ///
    /// # Notes
    /// * Requires a valid roblosecurity.
    /// * The amount of items that can be requested at once is unknown as not enough non-tradable limiteds exist, and the
    ///   endpoint doesn't accept duplicates.
    /// * Will repeat once if the x-csrf-token is invalid.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [X-CSRF-TOKEN Required Errors](#x-csrf-token-required-errors).
    /// * All errors under [Auth Required Errors](#auth-required-errors).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .roblosecurity(ROBLOSECURITY.to_string())
    ///     .build();
    ///
    /// let collectible_item_id_1 = "a4b5cb79-5218-4ca1-93fa-1e3436f595ef".to_owned();
    /// let collectible_item_id_2 = "61f2e366-9fe6-4562-8ce3-47334083372a".to_owned();
    /// let items = vec![collectible_item_id_1, collectible_item_id_2];
    ///
    /// let collectible_product_ids = client.collectible_product_id_bulk(items).await?;
    ///
    /// println!("Collectible Product ID 1: {}", collectible_product_ids[0]);
    /// println!("Collectible Product ID 2: {}", collectible_product_ids[1]);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collectible_product_id_bulk(
        &self,
        collectible_item_ids: Vec<String>,
    ) -> Result<Vec<String>, RoboatError> {
        let collectible_item_ids_len = collectible_item_ids.len();

        let details = self
            .non_tradable_limited_details(collectible_item_ids)
            .await?;

        let collectible_product_ids = details
            .iter()
            .map(|x| x.collectible_product_id.clone())
            .collect::<Vec<String>>();

        if collectible_product_ids.len() != collectible_item_ids_len {
            return Err(RoboatError::MalformedResponse);
        }

        Ok(collectible_product_ids)
    }

    /// Fetches the id of the original creator of a non-tradable limited. This is used when buying stock
    /// of an item (and not resellers).
    ///
    /// Uses [`Client::non_tradable_limited_details`] internally
    /// (which fetches from <https://apis.roblox.com/marketplace-items/v1/items/details>)
    ///
    /// # Notes
    /// * Requires a valid roblosecurity.
    /// * Will repeat once if the x-csrf-token is invalid.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [X-CSRF-TOKEN Required Errors](#x-csrf-token-required-errors).
    /// * All errors under [Auth Required Errors](#auth-required-errors).
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new()
    ///     .roblosecurity(ROBLOSECURITY.to_string())
    ///     .build();
    ///
    /// let collectible_item_id = "a4b5cb79-5218-4ca1-93fa-1e3436f595ef".to_owned();
    /// let collectible_creator_id = client.collectible_creator_id(collectible_item_id).await?;
    ///
    /// println!("Collectible Creator ID: {}", collectible_creator_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn collectible_creator_id(
        &self,
        collectible_item_id: String,
    ) -> Result<u64, RoboatError> {
        let details = self
            .non_tradable_limited_details(vec![collectible_item_id])
            .await?;

        let collectible_creator_id = details
            .first()
            .ok_or(RoboatError::MalformedResponse)?
            .creator_id;

        Ok(collectible_creator_id)
    }

    /// Purchases a non-tradable limited (includes ugc limiteds) using endpoint
    /// <https://apis.roblox.com/marketplace-sales/v1/item/{collectible_item_id}/purchase-item>.
    ///
    /// # Warning
    /// This endpoint and related endpoints may change as new things are discovered about this endpoint.
    /// This is because no resellers can sell items yet.
    ///
    /// # Notes
    /// * Requires a valid roblosecurity.
    /// * Will repeat once if the x-csrf-token is invalid.
    /// * Currently only tested to work when buying from users (as opposed to groups), and only tested
    ///   when buying the items from the original seller (with original stock). This is because
    ///   these are the only conditions that currently exist as of 4/14/2023.
    ///
    /// # Return Value Notes
    /// * Will return `Ok(())` if the limited was successfully purchased.
    ///
    /// # Argument Notes
    /// * `collectible_item_id` is the string id of a non-tradable limited. It can be
    ///   fetched using [`Client::collectible_item_id`].
    /// * `collectible_product_id` is the string product id of a non-tradable limited. It can be
    ///   fetched using [`Client::collectible_product_id`].
    /// * `collectible_seller_id` is the user id of the seller of a non-tradable limited. It can be
    ///   fetched using [`Client::collectible_creator_id`] (currently it is unknown how to buy from a reseller
    ///   instead of the original creator as they do not exist yet).
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [Auth Required Errors](#auth-required-errors).
    /// * All errors under [X-CSRF-TOKEN Required Errors](#x-csrf-token-required-errors).
    /// * [`RoboatError::PurchaseNonTradableLimitedError`] - Nested inside this error, all variants of [`PurchaseNonTradableLimitedError`] may be thrown.
    ///
    /// # Examples
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().roblosecurity(ROBLOSECURITY.to_string()).build();
    ///
    /// let collectible_item_id = "abc".to_string();;
    /// let collectible_product_id = "xyz".to_string();
    /// let collectible_seller_id = 123456789;
    /// let price = 0;
    ///
    /// let _ = client.purchase_non_tradable_limited(collectible_item_id, collectible_product_id, collectible_seller_id, price).await?;
    /// println!("Successfully Purchased!");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn purchase_non_tradable_limited(
        &self,
        collectible_item_id: String,
        collectible_product_id: String,
        collectible_seller_id: u64,
        price: u64,
    ) -> Result<(), RoboatError> {
        match self
            .purchase_non_tradable_limited_internal(
                collectible_item_id.clone(),
                collectible_product_id.clone(),
                collectible_seller_id,
                price,
            )
            .await
        {
            Ok(x) => Ok(x),
            Err(e) => match e {
                RoboatError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;

                    self.purchase_non_tradable_limited_internal(
                        collectible_item_id,
                        collectible_product_id,
                        collectible_seller_id,
                        price,
                    )
                    .await
                }
                _ => Err(e),
            },
        }
    }

    /// Fetches detailed information about a specific asset using its asset ID.
    ///
    /// This function retrieves asset information such as the asset's name, description,
    /// and other related details from the Roblox API. It uses the internal `get_asset_info_internal`
    /// function to send a request to the server and parse the response.
    ///
    /// # Argument Notes
    ///
    /// * `asset_id`: The unique ID of the asset whose information is being fetched. This ID can be obtained
    ///   through various means, such as browsing the asset page on Roblox or using other API endpoints.
    ///
    /// # Notes
    /// * requires .ROBLOSECURITY cookie
    /// * Will repeat once if the x-csrf-token is invalid.
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [Auth Required Errors](#auth-required-errors).
    /// * All errors under [X-CSRF-TOKEN Required Errors](#x-csrf-token-required-errors).
    /// * [`RoboatError::MalformedResponse`] - If the response from the server is malformed and cannot be parsed.
    /// * [`RoboatError::NetworkError`] - If there is an issue with the network request.
    /// * [`RoboatError::RequestError`] - If there is any general error with the request or the server response.
    ///
    /// # Example
    /// ```no_run
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().roblosecurity(ROBLOSECURITY.to_string()).build();
    ///
    /// let asset_id = 123456789;
    ///
    /// let asset_info = client.get_asset_info(asset_id).await?;
    /// println!("Asset Info: {:?}", asset_info);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_asset_info(&self, asset_id: u64) -> Result<AssetInfo, RoboatError> {
        match self.get_asset_info_internal(asset_id).await {
            Ok(x) => Ok(x),
            Err(e) => match e {
                RoboatError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;
                    self.get_asset_info_internal(asset_id).await
                }
                _ => Err(e),
            },
        }
    }

    /// Polls an Open Cloud asset upload operation using the endpoint
    /// <https://apis.roblox.com/assets/v1/operations/{operationId}>.
    ///
    /// Upload endpoints return an operation path rather than the final asset.
    /// Call this method repeatedly until `done == true`.
    ///
    /// # Notes
    /// * Requires an api key set with [`crate::ClientBuilder::api_key`], with the `asset:read` scope.
    ///   A roblosecurity is not needed.
    ///
    /// # Argument Notes
    /// * `path` - Operation path returned from an upload request (`operations/{operationId}`).
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [Api Key Required Errors](#api-key-required-errors).
    /// * [`RoboatError::MalformedResponse`] - If the response from the server cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::bedev2::AssetCreator;
    /// use roboat::catalog::AssetType;
    /// use roboat::ClientBuilder;
    /// use bytes::Bytes;
    ///
    /// const API_KEY: &str = "your_api_key";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().api_key(API_KEY.to_string()).build();
    ///
    /// let animation_data = Bytes::from(std::fs::read("animation.rbxm")?);
    ///
    /// let upload = client
    ///     .upload_asset(
    ///         AssetCreator::User(123456789),
    ///         "My Animation".to_string(),
    ///         "An example animation.".to_string(),
    ///         AssetType::Animation,
    ///         &animation_data,
    ///     )
    ///     .await?;
    ///
    /// loop {
    ///     let operation = client
    ///         .poll_upload_operation(upload.path.clone())
    ///         .await?;
    ///
    ///     if operation.done {
    ///         println!("New Asset ID: {:?}", operation.response.and_then(|x| x.asset_id));
    ///         break;
    ///     }
    ///
    ///     tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn poll_upload_operation(
        &self,
        path: String,
    ) -> Result<request_types::UploadAssetRaw, RoboatError> {
        self.poll_upload_operation_internal(path).await
    }

    /// Uploads an animation asset using the Open Cloud API endpoint
    /// <https://apis.roblox.com/assets/v1/assets>.
    ///
    /// This is the same as calling [`Client::upload_asset`] with [`AssetType::Animation`].
    ///
    /// # Notes
    /// * Requires an api key set with [`crate::ClientBuilder::api_key`], with the `asset:read` and
    ///   `asset:write` scopes. A roblosecurity is not needed.
    /// * `creator` is who the animation is uploaded under. It must be the owner of the API key,
    ///   or a group the owner can upload to.
    /// * `animation_data` must be the raw bytes of a valid `.rbxm` or `.rbxmx` animation file.
    /// * The upload is not finished when this returns. Use [`Client::poll_upload_operation`]
    ///   with the returned path to get the new asset id.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [Api Key Required Errors](#api-key-required-errors).
    /// * [`RoboatError::UnsupportedAsset`] - If `animation_data` isn't a `.rbxm` or `.rbxmx` file.
    /// * [`RoboatError::MalformedResponse`] - If the response from the server cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::bedev2::AssetCreator;
    /// use roboat::ClientBuilder;
    /// use bytes::Bytes;
    ///
    /// const API_KEY: &str = "your_api_key";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().api_key(API_KEY.to_string()).build();
    ///
    /// let animation_data = Bytes::from(std::fs::read("animation.rbxm")?);
    ///
    /// let upload = client.upload_animation(
    ///     AssetCreator::Group(123456789),
    ///     "My Animation".to_string(),
    ///     "An example animation.".to_string(),
    ///     &animation_data,
    /// ).await?;
    ///
    /// println!("Upload started: {}", upload.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_animation(
        &self,
        creator: AssetCreator,
        name: String,
        description: String,
        animation_data: &Bytes,
    ) -> Result<request_types::UploadAssetRaw, RoboatError> {
        self.upload_asset(
            creator,
            name,
            description,
            AssetType::Animation,
            animation_data,
        )
        .await
    }

    /// Uploads an asset using the Open Cloud API endpoint
    /// <https://apis.roblox.com/assets/v1/assets>.
    ///
    /// The file format is detected from `asset_data`, so the raw bytes of a downloaded
    /// asset (for example from [`Client::fetch_cloud_asset_data`]) can be passed directly.
    ///
    /// # Notes
    /// * Requires an api key set with [`crate::ClientBuilder::api_key`], with the `asset:read` and
    ///   `asset:write` scopes. A roblosecurity is not needed.
    /// * `creator` is who the asset is uploaded under. It must be the owner of the API key,
    ///   or a group the owner can upload to.
    /// * The upload is not finished when this returns. Use [`Client::poll_upload_operation`]
    ///   with the returned path to get the new asset id.
    ///
    /// # Argument Notes
    /// * `asset_type` must be one of the types supported by Open Cloud:
    ///   * [`AssetType::Animation`] - `.rbxm`, `.rbxmx`
    ///   * [`AssetType::Audio`] - `.mp3`, `.ogg`, `.wav`, `.flac`
    ///   * [`AssetType::Decal`] - `.png`, `.jpeg`, `.bmp`
    ///   * [`AssetType::Model`] - `.rbxm`, `.rbxmx`, `.fbx`, `.glb`
    ///   * [`AssetType::Video`] - `.mp4`, `.mov`
    /// * Images ([`AssetType::Image`]) are uploaded as a [`AssetType::Decal`]. The returned asset id
    ///   is the decal, not the image inside it.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [Api Key Required Errors](#api-key-required-errors).
    /// * [`RoboatError::UnsupportedAsset`] - If the asset type or file format can't be uploaded.
    /// * [`RoboatError::MalformedResponse`] - If the response from the server cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::bedev2::AssetCreator;
    /// use roboat::catalog::AssetType;
    /// use roboat::ClientBuilder;
    /// use bytes::Bytes;
    ///
    /// const API_KEY: &str = "your_api_key";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().api_key(API_KEY.to_string()).build();
    ///
    /// let audio_data = Bytes::from(std::fs::read("sound.ogg")?);
    ///
    /// let upload = client.upload_asset(
    ///     AssetCreator::User(123456789),
    ///     "My Sound".to_string(),
    ///     "An example sound.".to_string(),
    ///     AssetType::Audio,
    ///     &audio_data,
    /// ).await?;
    ///
    /// println!("Upload started: {}", upload.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_asset(
        &self,
        creator: AssetCreator,
        name: String,
        description: String,
        asset_type: AssetType,
        asset_data: &Bytes,
    ) -> Result<request_types::UploadAssetRaw, RoboatError> {
        self.cloud_upload_asset_internal(creator, name, description, asset_type, asset_data)
            .await
    }

    /// Updates the name and/or description of an asset using the Open Cloud API endpoint
    /// <https://apis.roblox.com/assets/v1/assets/{assetId}>.
    ///
    /// Only the fields that are `Some` get updated. The asset content isn't changed.
    ///
    /// # Notes
    /// * Requires an api key set with [`crate::ClientBuilder::api_key`], with the `asset:read` and
    ///   `asset:write` scopes. A roblosecurity is not needed.
    /// * The update is not finished when this returns. Use [`Client::poll_upload_operation`]
    ///   with the returned path to wait for it.
    /// * Roblox doesn't reject names it doesn't allow, it replaces them with `#`. This can be
    ///   used to rename an asset whose name got filtered.
    ///
    /// # Errors
    /// * All errors under [Standard Errors](#standard-errors).
    /// * All errors under [Api Key Required Errors](#api-key-required-errors).
    /// * [`RoboatError::BadRequest`] - If both `display_name` and `description` are `None`.
    /// * [`RoboatError::MalformedResponse`] - If the response from the server cannot be parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use roboat::catalog::AssetType;
    /// use roboat::ClientBuilder;
    ///
    /// const API_KEY: &str = "your_api_key";
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = ClientBuilder::new().api_key(API_KEY.to_string()).build();
    ///
    /// let update = client
    ///     .update_asset_details(
    ///         123456789,
    ///         AssetType::Animation,
    ///         Some("New Name".to_string()),
    ///         None,
    ///     )
    ///     .await?;
    ///
    /// println!("Update started: {}", update.path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_asset_details(
        &self,
        asset_id: u64,
        asset_type: AssetType,
        display_name: Option<String>,
        description: Option<String>,
    ) -> Result<request_types::UploadAssetRaw, RoboatError> {
        self.update_asset_details_internal(asset_id, asset_type, display_name, description)
            .await
    }

    /// Uploads classic clothing to a group. This currently only works for classic clothing and
    /// for people uploading from groups. This is because the endpoint is not fully understood yet
    /// and reverse engineering it is expensive.
    ///
    /// Uses endpoint <https://apis.roblox.com/assets/user-auth/v1/assets>.
    ///
    /// # WARNING: UNDER CONSTRUCTION
    ///
    /// # Notes
    /// * Requires a valid roblosecurity.
    /// * Will repeat once if the x-csrf-token is invalid.
    /// * The `image_path` must be a valid path to an image file.
    pub async fn upload_classic_clothing_to_group(
        &self,
        group_id: u64,
        name: String,
        description: String,
        image_path: String,
        classic_clothing_type: ClassicClothingType,
    ) -> Result<(), RoboatError> {
        match self
            .upload_classic_clothing_to_group_internal(
                group_id,
                name.clone(),
                description.clone(),
                image_path.clone(),
                classic_clothing_type,
            )
            .await
        {
            Ok(x) => Ok(x),
            Err(e) => match e {
                RoboatError::InvalidXcsrf(new_xcsrf) => {
                    self.set_xcsrf(new_xcsrf).await;

                    self.upload_classic_clothing_to_group_internal(
                        group_id,
                        name,
                        description,
                        image_path,
                        classic_clothing_type,
                    )
                    .await
                }
                _ => Err(e),
            },
        }
    }
}

mod internal {
    use std::path::Path;

    use bytes::Bytes;
    use reqwest::header;

    use super::{
        cloud_upload_file_format, request_types, sort_items_by_argument_order, AssetCreator,
        ClassicClothingType, NonTradableLimitedDetails, PurchaseNonTradableLimitedError,
        COLLECTIBLE_ITEM_DETAILS_API, OPEN_CLOUD_ASSET_API, OPEN_CLOUD_OPERATION_API,
        PURCHASE_NON_TRADEABLE_LIMITED_API_PART_1, PURCHASE_NON_TRADEABLE_LIMITED_API_PART_2,
    };
    use crate::{
        bedev2::{request_types::AssetInfo, ASSET_API},
        catalog, Client, RoboatError, API_KEY_HEADER, XCSRF_HEADER,
    };

    impl Client {
        pub(super) async fn non_tradable_limited_details_internal(
            &self,
            collectible_item_ids: Vec<String>,
        ) -> Result<Vec<NonTradableLimitedDetails>, RoboatError> {
            let request_body = serde_json::json!({
                "itemIds": collectible_item_ids,
            });

            let request_result = self
                .reqwest_client
                .post(COLLECTIBLE_ITEM_DETAILS_API)
                .header(XCSRF_HEADER, self.xcsrf().await)
                .header(header::COOKIE, self.cookie_string()?)
                .json(&request_body)
                .send()
                .await;

            let response = Self::validate_request_result(request_result).await?;
            let raw =
                Self::parse_to_raw::<Vec<request_types::NonTradableLimitedDetailsRaw>>(response)
                    .await?;

            let mut collectible_item_details = Vec::new();

            for raw_details in raw {
                let details = NonTradableLimitedDetails::try_from(raw_details)?;
                collectible_item_details.push(details);
            }

            sort_items_by_argument_order(&mut collectible_item_details, &collectible_item_ids);

            Ok(collectible_item_details)
        }

        pub(super) async fn purchase_non_tradable_limited_internal(
            &self,
            collectible_item_id: String,
            collectible_product_id: String,
            seller_id: u64,
            price: u64,
        ) -> Result<(), RoboatError> {
            let idempotency_key = uuid::Uuid::new_v4().to_string();
            let client_user_id = self.user_id().await?;

            let request_body = serde_json::json!({
                "collectibleItemId": collectible_item_id,
                "expectedCurrency": 1,
                "expectedPrice": price,
                "expectedPurchaserId":client_user_id,
                "expectedPurchaserType": "User",
                "expectedSellerId": seller_id,
                "expectedSellerType": "User",
                "idempotencyKey": idempotency_key,
                "collectibleProductId": collectible_product_id,
            });

            let formatted_url = format!(
                "{}{}{}",
                PURCHASE_NON_TRADEABLE_LIMITED_API_PART_1,
                collectible_item_id,
                PURCHASE_NON_TRADEABLE_LIMITED_API_PART_2
            );

            let request_result = self
                .reqwest_client
                .post(formatted_url)
                .header(XCSRF_HEADER, self.xcsrf().await)
                .header(header::COOKIE, self.cookie_string()?)
                .json(&request_body)
                .send()
                .await;

            let response = Self::validate_request_result(request_result).await?;
            let raw = Self::parse_to_raw::<request_types::PurchaseNonTradeableLimitedRaw>(response)
                .await?;

            if raw.purchased {
                return Ok(());
            }

            let err_msg = raw.error_message.ok_or(RoboatError::MalformedResponse)?;

            match err_msg.as_str() {
                "PriceMismatch" => Err(RoboatError::PurchaseNonTradableLimitedError(
                    PurchaseNonTradableLimitedError::PriceMismatch,
                )),
                "QuantityExhausted" => Err(RoboatError::PurchaseNonTradableLimitedError(
                    PurchaseNonTradableLimitedError::SoldOut,
                )),
                _ => Err(RoboatError::PurchaseNonTradableLimitedError(
                    PurchaseNonTradableLimitedError::UnknownRobloxErrorMsg(raw.purchase_result),
                )),
            }
        }

        pub(super) async fn get_asset_info_internal(
            &self,
            asset_id: u64,
        ) -> Result<AssetInfo, RoboatError> {
            let cookie_string = self.cookie_string()?;
            let xcsrf = self.xcsrf().await;
            let formatted_url = format!("{}/{}", ASSET_API, asset_id);

            let response_result = self
                .reqwest_client
                .get(formatted_url)
                .header(header::COOKIE, cookie_string)
                .header(XCSRF_HEADER, xcsrf)
                .send()
                .await;

            let response = Self::validate_request_result(response_result).await?;

            let asset_info = Self::parse_to_raw::<request_types::AssetInfo>(response).await?;

            Ok(asset_info)
        }

        pub(super) async fn poll_upload_operation_internal(
            &self,
            path: String,
        ) -> Result<request_types::UploadAssetRaw, RoboatError> {
            // The path from an upload response looks like "operations/{operationId}"
            let operation_id = path.trim_start_matches("operations/");
            let url = format!("{}/{}", OPEN_CLOUD_OPERATION_API, operation_id);

            let response_result = self
                .reqwest_client
                .get(url)
                .header(API_KEY_HEADER, self.api_key()?)
                .send()
                .await;

            let response = Self::validate_request_result(response_result)
                .await
                .map_err(Self::api_key_error)?;
            let raw = Self::parse_to_raw::<request_types::UploadAssetRaw>(response).await?;
            Ok(raw)
        }

        pub(super) async fn cloud_upload_asset_internal(
            &self,
            creator: AssetCreator,
            name: String,
            description: String,
            asset_type: catalog::AssetType,
            asset_data: &Bytes,
        ) -> Result<request_types::UploadAssetRaw, RoboatError> {
            let creator = match creator {
                AssetCreator::User(id) => serde_json::json!({ "userId": id.to_string() }),
                AssetCreator::Group(id) => serde_json::json!({ "groupId": id.to_string() }),
            };

            let (file_name, mime_type) = cloud_upload_file_format(asset_type, asset_data)?;

            let request_body = serde_json::json!({
                "displayName": name,
                "description": description,
                "assetType": asset_type.to_string(),
                "creationContext": {
                    "creator": creator,
                    "expectedPrice": 0,
                },
            });

            let form = reqwest::multipart::Form::new()
                .part(
                    "fileContent",
                    reqwest::multipart::Part::bytes(asset_data.to_vec())
                        .file_name(file_name)
                        .mime_str(mime_type)
                        .map_err(|_| RoboatError::MalformedResponse)?,
                )
                .text("request", request_body.to_string());

            let response_result = self
                .reqwest_client
                .post(OPEN_CLOUD_ASSET_API)
                .header(API_KEY_HEADER, self.api_key()?)
                .multipart(form)
                .send()
                .await;

            let response = Self::validate_request_result(response_result)
                .await
                .map_err(Self::api_key_error)?;
            let raw = Self::parse_to_raw::<request_types::UploadAssetRaw>(response).await?;
            Ok(raw)
        }

        pub(super) async fn update_asset_details_internal(
            &self,
            asset_id: u64,
            asset_type: catalog::AssetType,
            display_name: Option<String>,
            description: Option<String>,
        ) -> Result<request_types::UploadAssetRaw, RoboatError> {
            let mut update_mask = Vec::new();
            let mut request_body = serde_json::json!({
                "assetId": asset_id,
                "assetType": asset_type.to_string(),
            });

            if let Some(display_name) = display_name {
                update_mask.push("displayName");
                request_body["displayName"] = serde_json::json!(display_name);
            }
            if let Some(description) = description {
                update_mask.push("description");
                request_body["description"] = serde_json::json!(description);
            }
            if update_mask.is_empty() {
                return Err(RoboatError::BadRequest);
            }

            let form = reqwest::multipart::Form::new().text("request", request_body.to_string());
            let formatted_url = format!("{}/{}", OPEN_CLOUD_ASSET_API, asset_id);

            let response_result = self
                .reqwest_client
                .patch(formatted_url)
                .query(&[("updateMask", update_mask.join(","))])
                .header(API_KEY_HEADER, self.api_key()?)
                .multipart(form)
                .send()
                .await;

            let response = Self::validate_request_result(response_result)
                .await
                .map_err(Self::api_key_error)?;
            let raw = Self::parse_to_raw::<request_types::UploadAssetRaw>(response).await?;
            Ok(raw)
        }

        pub(super) async fn upload_classic_clothing_to_group_internal(
            &self,
            group_id: u64,
            name: String,
            description: String,
            image_path: String,
            classic_clothing_type: ClassicClothingType,
        ) -> Result<(), RoboatError> {
            let filename = Path::new(&image_path)
                .file_name()
                .map(|x| x.to_string_lossy().to_string())
                .ok_or(RoboatError::InvalidPath(image_path.clone()))?;

            let asset_name_patch = match classic_clothing_type {
                ClassicClothingType::Shirt => "Shirt",
                ClassicClothingType::Pants => "Pants",
                ClassicClothingType::TShirt => "Tshirt",
            };

            let expected_price = match classic_clothing_type {
                ClassicClothingType::Shirt | ClassicClothingType::Pants => 10,
                ClassicClothingType::TShirt => 0,
            };

            let form = reqwest::multipart::Form::new()
                .part("fileContent", reqwest::multipart::Part::bytes(tokio::fs::read(image_path).await?).file_name(filename))
                .text("request", format!("{{\"displayName\":\"{name}\",\"description\":\"{description}\",\"assetType\":\"{asset_name_patch}\",\"creationContext\":{{\"creator\":{{\"groupId\":{group_id}}},\"expectedPrice\":{expected_price}}}}}" ));

            let cookie_string = self.cookie_string()?;
            let xcsrf = self.xcsrf().await;

            let response_result = self
                .reqwest_client
                .request(reqwest::Method::POST, ASSET_API)
                .header(header::COOKIE, cookie_string)
                .header(XCSRF_HEADER, xcsrf)
                .multipart(form)
                .send()
                .await;

            let response = Self::validate_request_result(response_result).await?;
            let _ = Self::parse_to_raw::<request_types::UploadClassicClothingRaw>(response).await?;

            Ok(())
        }
    }
}

/// Gets the file name and mime type that the Open Cloud upload endpoint expects
/// for an asset, by checking the asset type and the first bytes of the file.
///
/// For example, an [`AssetType::Audio`] that starts with `OggS` returns `("asset.ogg", "audio/ogg")`.
fn cloud_upload_file_format(
    asset_type: AssetType,
    asset_data: &[u8],
) -> Result<(&'static str, &'static str), RoboatError> {
    let is_rbxm = asset_data.starts_with(b"<roblox!");
    let is_rbxmx = !is_rbxm && asset_data.starts_with(b"<roblox");
    let is_mp4 = asset_data.get(4..8) == Some(b"ftyp");

    let format = match asset_type {
        AssetType::Animation | AssetType::Model if is_rbxm => ("asset.rbxm", "model/x-rbxm"),
        AssetType::Animation | AssetType::Model if is_rbxmx => ("asset.rbxmx", "model/x-rbxmx"),
        AssetType::Model if asset_data.starts_with(b"glTF") => ("asset.glb", "model/gltf-binary"),
        AssetType::Model if asset_data.starts_with(b"Kaydara FBX Binary") => {
            ("asset.fbx", "model/fbx")
        }
        AssetType::Audio if asset_data.starts_with(b"OggS") => ("asset.ogg", "audio/ogg"),
        AssetType::Audio if asset_data.starts_with(b"fLaC") => ("asset.flac", "audio/flac"),
        AssetType::Audio
            if asset_data.starts_with(b"RIFF") && asset_data.get(8..12) == Some(b"WAVE") =>
        {
            ("asset.wav", "audio/wav")
        }
        AssetType::Audio
            if asset_data.starts_with(b"ID3")
                || (asset_data.len() > 1
                    && asset_data[0] == 0xFF
                    && asset_data[1] & 0xE0 == 0xE0) =>
        {
            ("asset.mp3", "audio/mpeg")
        }
        AssetType::Decal if asset_data.starts_with(b"\x89PNG") => ("asset.png", "image/png"),
        AssetType::Decal if asset_data.starts_with(&[0xFF, 0xD8, 0xFF]) => {
            ("asset.jpeg", "image/jpeg")
        }
        AssetType::Decal if asset_data.starts_with(b"BM") => ("asset.bmp", "image/bmp"),
        AssetType::Video if is_mp4 && asset_data.get(8..12) == Some(b"qt  ") => {
            ("asset.mov", "video/mov")
        }
        AssetType::Video if is_mp4 => ("asset.mp4", "video/mp4"),
        AssetType::Animation
        | AssetType::Model
        | AssetType::Audio
        | AssetType::Decal
        | AssetType::Video => {
            return Err(RoboatError::UnsupportedAsset(format!(
                "file format for {}",
                asset_type
            )))
        }
        _ => {
            return Err(RoboatError::UnsupportedAsset(format!(
                "type {}",
                asset_type
            )))
        }
    };

    Ok(format)
}

/// Makes sure that the items are in the same order as the arguments.
///
/// For example, if the arguments are `["1", "2", "3"]` and the resulting items are `["2", "1", "3"]`,
/// then the resulting items will be `["1", "2", "3"]`.
fn sort_items_by_argument_order(items: &mut [NonTradableLimitedDetails], arguments: &[String]) {
    items.sort_by(|a, b| {
        let a_index = arguments
            .iter()
            .position(|collectible_item_id| collectible_item_id == &a.collectible_item_id)
            .unwrap_or(usize::MAX);

        let b_index = arguments
            .iter()
            .position(|collectible_item_id| collectible_item_id == &b.collectible_item_id)
            .unwrap_or(usize::MAX);

        a_index.cmp(&b_index)
    });
}
