use crate::Error;
use async_trait::async_trait;
use oauth2::{AccessToken, RefreshToken, StandardDeviceAuthorizationResponse};

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct TokenPair {
    pub access_token_data: AccessTokenData,
    pub refresh_token: Option<RefreshToken>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AccessTokenData {
    pub token: AccessToken,
    #[serde(with = "chrono::serde::ts_seconds_option")]
    pub expires_in: Option<chrono::DateTime<chrono::Utc>>,
}

#[async_trait]
pub trait OauthClientDeviceFlow {
    async fn device_code(&self) -> Result<StandardDeviceAuthorizationResponse, Error>;
    async fn register_device(
        &self,
        device_auth_res: StandardDeviceAuthorizationResponse,
    ) -> Result<TokenPair, Error>;
}
