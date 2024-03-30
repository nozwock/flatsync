use self::client::GitHubClient;
use async_trait::async_trait;
use libflatsync_common::FlatpakInstallationPayload;
use log::debug;
use serde_json::json;
use zbus::zvariant::Type;

// '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
use super::{data_sink::DataSink, rest_client::RestClient};
use crate::{
    data_sinks::{data_sink::FILE_NAME, github::models::GistFile},
    Error,
};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

static GH_SINK_NAME: &str = "github-gists";
static GH_API_URL: &str = "https://api.github.com/gists";

pub mod client {
    use crate::data_sinks::data_sink_client::DataSinkClient;
    use crate::data_sinks::data_sink_client::SecretType;
    use crate::data_sinks::oauth_client::OauthClient;
    use crate::data_sinks::rest_client::RestClient;
    use crate::Error;
    use async_trait::async_trait;
    use libflatsync_common::providers::github::get_github_basic_client;
    use libflatsync_common::providers::oauth_client::{AccessTokenData, TokenPair};
    use oauth2::basic::BasicClient;
    use oauth2::reqwest::async_http_client;
    use oauth2::TokenResponse;
    // '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
    use reqwest::{IntoUrl, Method, RequestBuilder};

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    pub struct GitHubClient {
        request: RequestBuilder,
        client: BasicClient,
    }

    impl DataSinkClient for GitHubClient {
        fn sink_name(&self) -> &'static str {
            super::GH_SINK_NAME
        }
    }

    impl GitHubClient {
        pub async fn new<U: IntoUrl>(method: Method, url: U) -> Result<GitHubClient, Error> {
            Ok(GitHubClient {
                request: reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()
                    .unwrap()
                    .request(method, url),
                client: get_github_basic_client()?,
            })
        }

        async fn secret(&self) -> Result<SecretType, Error> {
            let token_pair = serde_json::from_str::<TokenPair>(&self.secret_raw().await?).unwrap();

            Ok(SecretType::OAuth(token_pair))
        }

        pub async fn send(self) -> Result<reqwest::Response, Error> {
            let SecretType::OAuth(token_pair) = self.secret().await? else {
                unreachable!();
            };

            self.check_tokens(&token_pair).await.unwrap();

            self.request
                .header("Accept", "application/vnd.github+json")
                .header(
                    "Authorization",
                    format!("Bearer {}", token_pair.access_token_data.token.secret()),
                )
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
                .map_err(Error::HttpFailure)
        }
    }

    #[async_trait]
    impl OauthClient for GitHubClient {
        fn oauth2_scopes(&self) -> Vec<String> {
            vec![]
        }

        async fn check_tokens(&self, tokens: &TokenPair) -> Result<TokenPair, Error> {
            let token_data = &tokens.access_token_data;

            if token_data.expires_in.is_none() {
                return Ok(tokens.clone());
            }

            if token_data.expires_in.unwrap() > chrono::Utc::now() {
                return Ok(tokens.clone());
            }

            let token = self
                .client
                .exchange_refresh_token(&tokens.refresh_token.clone().unwrap())
                .request_async(async_http_client)
                .await
                .unwrap();

            let token_data = AccessTokenData {
                token: token.access_token().clone(),
                expires_in: token
                    .expires_in()
                    .map(|e| chrono::Utc::now() + chrono::Duration::seconds(e.as_secs() as i64)),
            };

            Ok(TokenPair {
                access_token_data: token_data,
                refresh_token: Some(token.refresh_token().unwrap().clone()),
            })
        }
    }

    impl RestClient for GitHubClient {
        fn builder(&mut self) -> RequestBuilder {
            self.request.try_clone().unwrap()
        }

        fn set_builder(&mut self, builder: RequestBuilder) {
            self.request = builder;
        }
    }
}

mod models {
    use libflatsync_common::FlatpakInstallationPayload;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    // GitHub-Gists expects us to send the content of the file as a string, not as a JSON object.
    fn content_to_string<S>(
        f: &FlatpakInstallationPayload,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&serde_json::to_string_pretty(&f).unwrap())
    }

    fn string_to_content<'de, D>(deserializer: D) -> Result<FlatpakInstallationPayload, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        serde_json::from_str(&s).map_err(serde::de::Error::custom)
    }

    #[derive(Serialize, Deserialize)]
    pub struct GistFile {
        #[serde(serialize_with = "content_to_string")]
        #[serde(deserialize_with = "string_to_content")]
        pub content: FlatpakInstallationPayload,
    }
}

pub struct GitHubGistDataSink {}

impl GitHubGistDataSink {
    pub async fn new() -> Result<Self, Error> {
        Ok(Self {})
    }
}

#[async_trait]
impl DataSink for GitHubGistDataSink {
    async fn create(&self, payload: FlatpakInstallationPayload) -> Result<(), Error> {
        #[derive(Serialize, Deserialize, Type, Debug)]
        pub struct CreateGistResponse {
            pub id: String,
        }

        let mut client = GitHubClient::new(Method::POST, GH_API_URL).await?;

        client.body(json!({
            "description": "Installed Flatpaks and its remote repositories",
            "public": false,
            "files": {
                FILE_NAME: GistFile { content: payload }
            },
        }));

        let resp = client.send().await?;
        debug!("Gist creation response: {:?}", resp);
        let data: CreateGistResponse = resp.json().await?;
        debug!("Gist creation response data: {:?}", data);
        self.set_sink_id(&data.id);
        Ok(())
    }

    async fn fetch(&self) -> Result<FlatpakInstallationPayload, Error> {
        // See https://docs.github.com/en/rest/gists/gists?apiVersion=2022-11-28#get-a-gist
        #[derive(Deserialize)]
        pub struct GetGistResponse {
            files: BTreeMap<String, GistFile>,
        }

        let mut resp: GetGistResponse =
            GitHubClient::new(Method::GET, format!("{}/{}", GH_API_URL, self.sink_id()))
                .await?
                .send()
                .await?
                .json()
                .await?;

        Ok(resp
            .files
            .remove(FILE_NAME)
            .ok_or(Error::MissingGistFiles)?
            .content)
    }

    async fn update(&self, payload: FlatpakInstallationPayload) -> Result<(), Error> {
        let mut client =
            GitHubClient::new(Method::POST, format!("{}/{}", GH_API_URL, self.sink_id())).await?;
        client.body(json!({
            "files": {
                FILE_NAME: GistFile { content: payload }
            },
        }));
        client.send().await?;

        Ok(())
    }

    fn sink_name(&self) -> &'static str {
        GH_SINK_NAME
    }
}
