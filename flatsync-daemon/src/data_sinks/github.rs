use self::client::GitHubClient;
use async_trait::async_trait;
use libflatsync_common::FlatpakInstallationPayload;
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

mod client {
    use crate::data_sinks::rest_client::RestClient;
    // '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
    use reqwest::{IntoUrl, Method, RequestBuilder};

    static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

    pub struct GitHubClient {
        request: RequestBuilder,
    }

    impl GitHubClient {
        pub fn new<U: IntoUrl>(method: Method, url: U) -> GitHubClient {
            GitHubClient {
                request: reqwest::Client::builder()
                    .user_agent(APP_USER_AGENT)
                    .build()
                    .unwrap()
                    .request(method, url),
            }
        }

        pub async fn send(self, github_token: &str) -> Result<reqwest::Response, reqwest::Error> {
            self.request
                .header("Accept", "application/vnd.github+json")
                .header("Authorization", format!("Bearer {}", github_token))
                .header("X-GitHub-Api-Version", "2022-11-28")
                .send()
                .await
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

pub struct GitHubGistDataSink {
    keyring: oo7::Keyring,
}

impl GitHubGistDataSink {
    pub async fn new() -> Result<Self, Error> {
        let keyring = oo7::Keyring::new().await?;
        Ok(Self { keyring })
    }
}

#[async_trait]
impl DataSink for GitHubGistDataSink {
    async fn create(&self, payload: FlatpakInstallationPayload) -> Result<(), Error> {
        #[derive(Serialize, Deserialize, Type)]
        pub struct CreateGistResponse {
            pub id: String,
        }

        let mut client = GitHubClient::new(Method::POST, "https://api.github.com/gists");

        client.body(json!({
            "description": "Installed Flatpaks and its remote repositories",
            "public": false,
            "files": {
                FILE_NAME: GistFile { content: payload }
            },
        }));

        let resp: CreateGistResponse = client.send(&self.secret().await?).await?.json().await?;
        self.set_sink_id(&resp.id);
        Ok(())
    }

    async fn fetch(&self) -> Result<FlatpakInstallationPayload, Error> {
        // See https://docs.github.com/en/rest/gists/gists?apiVersion=2022-11-28#get-a-gist
        #[derive(Deserialize)]
        pub struct GetGistResponse {
            files: BTreeMap<String, GistFile>,
        }

        let mut resp: GetGistResponse = GitHubClient::new(
            Method::GET,
            &format!("https://api.github.com/gists/{}", self.sink_id()),
        )
        .send(&self.secret().await?)
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
        let mut client = GitHubClient::new(
            Method::POST,
            format!("https://api.github.com/gists/{}", self.sink_id()),
        );
        client.body(json!({
            "files": {
                FILE_NAME: GistFile { content: payload }
            },
        }));
        client.send(&self.secret().await?).await?;

        Ok(())
    }

    fn sink_name(&self) -> &'static str {
        "github-gists"
    }

    fn keyring(&self) -> &oo7::Keyring {
        &self.keyring
    }
}
