use crate::users::ClientUserInformation;
use crate::RoboatError;
use reqwest::header::HeaderValue;
// We use tokio's version of rwlock so that readers to not starve writers on linux.
use tokio::sync::RwLock;

/// A client used for making requests to the Roblox API.
///
/// The client stores the roblosecurity cookie, Open Cloud api key, X-CSRF-TOKEN header, and an HTTPS client to send web
/// requests. The client also caches the user id, username, and display name of the user.
///
/// Constructed using a [`ClientBuilder`].
///
/// # Construction Examples
///
/// ## Without Roblosecurity or a Custom Reqwest Client
/// ```
/// use roboat::ClientBuilder;
///
/// let client = ClientBuilder::new().build();
/// ```
///
/// ## With a Roblosecurity
/// ```
/// use roboat::ClientBuilder;
///
/// const ROBLOSECURITY: &str = "roblosecurity";
///
/// let client = ClientBuilder::new().roblosecurity(ROBLOSECURITY.to_string()).build();
/// ```
///
/// ## With an Open Cloud Api Key
/// ```
/// use roboat::ClientBuilder;
///
/// const API_KEY: &str = "api_key";
///
/// let client = ClientBuilder::new().api_key(API_KEY.to_string()).build();
/// ```
///
/// ## With a Custom Reqwest Client
/// ```
/// use roboat::ClientBuilder;
///
/// let reqwest_client = reqwest::Client::new();
/// let client = ClientBuilder::new().reqwest_client(reqwest_client).build();
/// ```
///
/// ## With a Roblosecurity and a Custom Reqwest Client
/// ```
/// use roboat::ClientBuilder;
///
/// const ROBLOSECURITY: &str = "roblosecurity";
///
/// let reqwest_client = reqwest::Client::new();
/// let client = ClientBuilder::new().roblosecurity(ROBLOSECURITY.to_string()).reqwest_client(reqwest_client).build();
/// ```
///
/// # Standard Errors
/// The errors that can be returned by any of `Client`'s methods are:
/// - [`RoboatError::TooManyRequests`]
/// - [`RoboatError::InternalServerError`]
/// - [`RoboatError::BadRequest`]
/// - [`RoboatError::UnknownRobloxErrorCode`]
/// - [`RoboatError::UnidentifiedStatusCode`]
/// - [`RoboatError::ReqwestError`]
///
/// # Auth Required Errors
/// The errors that can be returned by any of `Client`'s methods that require authentication are:
/// - [`RoboatError::InvalidRoblosecurity`]
/// - [`RoboatError::RoblosecurityNotSet`]
///
/// # Api Key Required Errors
/// The errors that can be returned by any of `Client`'s methods that use Open Cloud and require an api key are:
/// - [`RoboatError::InvalidApiKey`]
/// - [`RoboatError::ApiKeyNotSet`]
///
/// # X-CSRF-TOKEN Required Errors
/// The errors that can be returned by any of `Client`'s methods that require the X-CSRF-TOKEN header are:
/// - [`RoboatError::InvalidXcsrf`]
/// - [`RoboatError::XcsrfNotReturned`]
///
/// # 2-Factor Authentication / Captcha Required Errors
/// The errors that can be returned by any of `Client`'s methods that require 2-factor authentication or a captcha are:
/// - [`RoboatError::ChallengeRequired`]
/// - [`RoboatError::UnknownStatus403Format`]
#[derive(Debug, Default)]
pub struct Client {
    /// The full cookie that includes the roblosecurity token.
    pub(crate) cookie_string: Option<HeaderValue>,
    /// The Open Cloud api key, sent in the x-api-key header.
    pub(crate) api_key: Option<HeaderValue>,
    /// The field holding the value for the X-CSRF-TOKEN header used in and returned by endpoints.
    pub(crate) xcsrf: RwLock<String>,
    /// Holds the user id, username, and display name of the user.
    pub(crate) user_information: RwLock<Option<ClientUserInformation>>,
    /// A Reqwest HTTP client used to send web requests.
    pub(crate) reqwest_client: reqwest::Client,
}

/// A builder used for constructing a [`Client`]. Constructed using [`ClientBuilder::new`].
#[derive(Clone, Debug, Default)]
pub struct ClientBuilder {
    roblosecurity: Option<String>,
    api_key: Option<String>,
    reqwest_client: Option<reqwest::Client>,
}

impl Client {
    /// Returns the user id of the user. If the user id is not cached, it will be fetched from Roblox first.
    ///
    /// The user id should be the only thing used to differentiate between accounts as
    /// username and display name can change.
    pub async fn user_id(&self) -> Result<u64, RoboatError> {
        let guard = self.user_information.read().await;
        let user_information_opt = &*guard;

        match user_information_opt {
            Some(user_information) => Ok(user_information.user_id),
            None => {
                // Drop the read lock as the writer lock will be requested.
                drop(guard);

                let user_info = self.user_information_internal().await?;
                Ok(user_info.user_id)
            }
        }
    }

    /// Returns the username of the user. If the username is not cached, it will be fetched from Roblox first.
    ///
    /// Username can change (although rarely). For this reason only user id should be used for differentiating accounts.
    pub async fn username(&self) -> Result<String, RoboatError> {
        let guard = self.user_information.read().await;
        let user_information_opt = &*guard;

        match user_information_opt {
            Some(user_information) => Ok(user_information.username.clone()),
            None => {
                // Drop the read lock as the writer lock will be requested.
                drop(guard);

                let user_info = self.user_information_internal().await?;
                Ok(user_info.username)
            }
        }
    }

    /// Returns the display name of the user. If the display name is not cached, it will be fetched from Roblox first.
    ///
    /// Display name can change. For this reason only user id should be used for differentiating accounts.
    pub async fn display_name(&self) -> Result<String, RoboatError> {
        let guard = self.user_information.read().await;
        let user_information_opt = &*guard;

        match user_information_opt {
            Some(user_information) => Ok(user_information.display_name.clone()),
            None => {
                // Drop the read lock as the writer lock will be requested.
                drop(guard);

                let user_info = self.user_information_internal().await?;
                Ok(user_info.display_name)
            }
        }
    }

    /// Used in [`Client::user_information_internal`]. This is implemented in the client
    /// module as we do not want other modules to have to interact with the rwlock directly.
    pub(crate) async fn set_user_information(&self, user_information: ClientUserInformation) {
        *self.user_information.write().await = Some(user_information);
    }

    /// Sets the xcsrf token of the client. Remember to .await this method.
    pub(crate) async fn set_xcsrf(&self, xcsrf: String) {
        *self.xcsrf.write().await = xcsrf;
    }

    /// Returns a copy of the xcsrf stored in the client. Remember to .await this method.
    pub(crate) async fn xcsrf(&self) -> String {
        self.xcsrf.read().await.clone()
    }

    /// Returns a copy of the cookie string stored in the client.
    /// If the roblosecurity has not been set, [`RoboatError::RoblosecurityNotSet`] is returned.
    pub(crate) fn cookie_string(&self) -> Result<HeaderValue, RoboatError> {
        let cookie_string_opt = &self.cookie_string;

        match cookie_string_opt {
            Some(cookie) => Ok(cookie.clone()),
            None => Err(RoboatError::RoblosecurityNotSet),
        }
    }

    /// Returns a copy of the api key header stored in the client.
    /// If the api key has not been set, [`RoboatError::ApiKeyNotSet`] is returned.
    pub(crate) fn api_key(&self) -> Result<HeaderValue, RoboatError> {
        let api_key_opt = &self.api_key;

        match api_key_opt {
            Some(api_key) => Ok(api_key.clone()),
            None => Err(RoboatError::ApiKeyNotSet),
        }
    }

    /// Open Cloud endpoints don't use a roblosecurity or an xcsrf, so the auth errors
    /// returned from them mean the api key was rejected.
    pub(crate) fn api_key_error(error: RoboatError) -> RoboatError {
        match error {
            RoboatError::InvalidRoblosecurity
            | RoboatError::XcsrfNotReturned
            | RoboatError::UnknownStatus403Format => RoboatError::InvalidApiKey,
            _ => error,
        }
    }
}

impl ClientBuilder {
    /// Creates a new [`ClientBuilder`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the roblosecurity for the client.
    ///
    /// # Example
    /// ```rust
    /// use roboat::ClientBuilder;
    ///
    /// const ROBLOSECURITY: &str = "roblosecurity";
    ///
    /// let client = ClientBuilder::new().roblosecurity(ROBLOSECURITY.to_string()).build();
    /// ```
    pub fn roblosecurity(mut self, roblosecurity: String) -> Self {
        self.roblosecurity = Some(roblosecurity);
        self
    }

    /// Sets the Open Cloud api key for the client. This is used by Open Cloud endpoints
    /// instead of the roblosecurity, and can be set alongside it.
    ///
    /// Api keys are made at <https://create.roblox.com/dashboard/credentials>. Unlike a roblosecurity,
    /// a key only has the scopes it was given, so check the docs of each method for the scopes it needs.
    ///
    /// # Example
    /// ```rust
    /// use roboat::ClientBuilder;
    ///
    /// const API_KEY: &str = "api_key";
    ///
    /// let client = ClientBuilder::new().api_key(API_KEY.to_string()).build();
    /// ```
    pub fn api_key(mut self, api_key: String) -> Self {
        self.api_key = Some(api_key);
        self
    }

    /// Sets the [`reqwest::Client`] for the client.
    ///
    /// # Example
    /// ```rust
    /// use roboat::ClientBuilder;
    ///
    /// let reqwest_client = reqwest::Client::new();
    /// let client = ClientBuilder::new().reqwest_client(reqwest_client).build();
    /// ```
    pub fn reqwest_client(mut self, reqwest_client: reqwest::Client) -> Self {
        self.reqwest_client = Some(reqwest_client);
        self
    }

    /// Builds the [`Client`]. This consumes the builder.
    ///
    /// # Example
    /// ```rust
    /// use roboat::ClientBuilder;
    ///
    /// let client = ClientBuilder::new().build();
    /// ```
    pub fn build(self) -> Client {
        Client {
            cookie_string: self
                .roblosecurity
                .as_ref()
                .map(|x| create_cookie_string_header(x)),
            api_key: self.api_key.as_ref().map(|x| create_api_key_header(x)),
            reqwest_client: self.reqwest_client.unwrap_or_default(),
            ..Default::default()
        }
    }
}

fn create_cookie_string_header(roblosecurity: &str) -> HeaderValue {
    // We panic here because I really really really hope that nobody is using invalid characters in their roblosecurity.
    let mut header = HeaderValue::from_str(&format!(".ROBLOSECURITY={}", roblosecurity))
        .expect("Invalid roblosecurity characters.");

    header.set_sensitive(true);

    header
}

fn create_api_key_header(api_key: &str) -> HeaderValue {
    // Same as the roblosecurity, we panic here as an api key should never have invalid characters.
    let mut header = HeaderValue::from_str(api_key).expect("Invalid api key characters.");

    header.set_sensitive(true);

    header
}
