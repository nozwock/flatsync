use async_trait::async_trait;
use log::error;
use oauth2::{
    basic::BasicClient, http, reqwest::async_http_client, AuthUrl, ClientId,
    DeviceAuthorizationUrl, HttpRequest, HttpResponse, StandardDeviceAuthorizationResponse,
    TokenResponse, TokenUrl,
};

use crate::Error;

use super::oauth_client::{AccessTokenData, OauthClientDeviceFlow, TokenPair};

static GH_CLIENT_ID: &str = "Iv1.1bf99f29c6b7d129";
static GH_AUTH_URL: &str = "https://github.com/login/oauth/authorize";
static GH_DEVICE_AUTH_URL: &str = "https://github.com/login/device/code";
static GH_TOKEN_URL: &str = "https://github.com/login/oauth/access_token";

// Specific to the behavior of GitHub Apps. We need the "Repository Metadata" permission to interact with the Gist API, and we only get that by installing the app the the user's account
pub static GH_APP_INSTALLATION_URL: &str = "https://github.com/apps/flatsync/installations/new";

async fn custom_async_http_client(request: HttpRequest) -> Result<HttpResponse, Error> {
    let orig_res = async_http_client(request).await;

    match orig_res {
        Err(e) => Err(Error::OAuth2ReqwestFailure(e.to_string())),
        Ok(mut res) => {
            let msg = std::str::from_utf8(&res.body).map_err(Error::other)?;

            // The oauth2 library expects a 400 error code, but GitHub returns a 200
            // See https://github.com/ramosbugs/oauth2-rs/issues/218#issuecomment-1575245490
            // Also https://www.rfc-editor.org/rfc/rfc8628#section-3.5
            if msg.contains("authorization_pending")
                || msg.contains("slow_down")
                || msg.contains("access_denied")
                || msg.contains("expired_token")
            {
                res.status_code = http::StatusCode::BAD_REQUEST;
            }
            Ok(res)
        }
    }
}

pub fn get_github_basic_client() -> anyhow::Result<BasicClient> {
    Ok(BasicClient::new(
        ClientId::new(GH_CLIENT_ID.into()),
        None,
        AuthUrl::new(GH_AUTH_URL.into())?,
        Some(TokenUrl::new(GH_TOKEN_URL.into())?),
    )
    .set_device_authorization_url(DeviceAuthorizationUrl::new(GH_DEVICE_AUTH_URL.into())?))
}

pub struct GitHubProvider {
    client: BasicClient,
}

#[async_trait]
impl OauthClientDeviceFlow for GitHubProvider {
    async fn device_code(&self) -> Result<StandardDeviceAuthorizationResponse, Error> {
        let req = self.client.exchange_device_code().map_err(Error::other)?;

        Ok(req
            .request_async(async_http_client)
            .await
            .map_err(Error::other)?)
    }

    async fn register_device(
        &self,
        device_auth_res: StandardDeviceAuthorizationResponse,
    ) -> Result<TokenPair, Error> {
        let token = self
            .client
            .exchange_device_access_token(&device_auth_res)
            .request_async(custom_async_http_client, tokio::time::sleep, None)
            .await;

        let token = match token {
            Ok(t) => t,
            Err(e) => {
                error!(
                    "Error when trying to authenticate against GitHub's API: {:?}",
                    e
                );
                return Err(Error::OAuth2ReqwestFailure(e.to_string()));
            }
        };

        let token_pair = TokenPair {
            access_token_data: AccessTokenData {
                token: token.access_token().clone(),
                expires_in: token
                    .expires_in()
                    .map(|d| chrono::Utc::now() + chrono::Duration::seconds(d.as_secs() as i64)),
            },
            refresh_token: token.refresh_token().cloned(),
        };

        Ok(token_pair)
    }
}

// impl Default for GitHubProvider {
//     fn default() -> Self {
//         Self::new()
//     }
// }

impl GitHubProvider {
    // NOTE: I see no other relevant error variant, so anyhow it is.
    // Not sure whether to change fn name to try_new
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            client: get_github_basic_client()?,
        })
    }
}
