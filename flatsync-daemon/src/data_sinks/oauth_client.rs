use crate::Error;
use libflatsync_common::providers::oauth_client::TokenPair;

pub trait OauthClient {
    fn oauth2_scopes(&self) -> Vec<String>;
    /// Checks access token for validity (i.e. expiry time).
    ///
    /// If the OAuth authorization server provides a refresh token, it is used to acquire a new token pair.
    ///
    /// We generally assume that a pair of refresh and access token is used,
    /// but if the service doesn't offer this, feel free to keep `refresh_token` a `None` value.
    ///
    /// # Error
    ///
    /// Errors out when no valid access token was found and/or no valid tokens could be acquired (i.e. expired/revoked refresh token).
    fn check_tokens(&self, tokens: &TokenPair) -> Result<TokenPair, Error>;
}
