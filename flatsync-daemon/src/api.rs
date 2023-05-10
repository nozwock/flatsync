use libflatsync_common::FlatpakInstallationMap;
// '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
use crate::Error;
use reqwest::{IntoUrl, Method, RequestBuilder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

static APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
static FILE_NAME: &str = "flatsync.json";

struct CustomClient {
    request: RequestBuilder,
}

impl CustomClient {
    pub fn new<U: IntoUrl>(method: Method, url: U) -> CustomClient {
        CustomClient {
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

    pub fn body<T: Serialize>(self, serializable: T) -> Self {
        let serialized = json!(serializable).to_string();
        CustomClient {
            request: self.request.body(serialized),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct GistFile {
    content: FlatpakInstallationMap,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGist {
    description: String,
    public: bool,
    files: BTreeMap<String, GistFile>,
}

#[derive(Debug)]
pub struct FetchGist {
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct FetchGistResponse {
    files: BTreeMap<String, FetchGistResponseFile>,
}

#[derive(Serialize, Deserialize)]
pub struct FetchGistResponseFile {
    filename: String,
    raw_url: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGistResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGist {
    files: BTreeMap<String, GistFile>,
}

#[derive(Serialize, Deserialize)]
pub struct GetGistResponse {
    files: BTreeMap<String, GistFile>,
}

impl CreateGist {
    pub fn new(description: String, public: bool, content: FlatpakInstallationMap) -> CreateGist {
        CreateGist {
            description,
            public,
            files: BTreeMap::from([(FILE_NAME.to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str) -> Result<CreateGistResponse, reqwest::Error> {
        CustomClient::new(Method::POST, "https://api.github.com/gists")
            .body(self)
            .send(github_token)
            .await?
            .json()
            .await
    }
}

impl UpdateGist {
    pub fn new(content: FlatpakInstallationMap) -> UpdateGist {
        UpdateGist {
            files: BTreeMap::from([(FILE_NAME.to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str, gist_id: &str) -> Result<(), reqwest::Error> {
        CustomClient::new(
            Method::POST,
            &format!("https://api.github.com/gists/{}", gist_id),
        )
        .body(self)
        .send(github_token)
        .await?;

        Ok(())
    }
}

impl FetchGist {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        Self {
            id: id.as_ref().into(),
        }
    }

    pub async fn fetch<S: AsRef<str>>(&self, gh_token: S) -> Result<FlatpakInstallationMap, Error> {
        // See https://docs.github.com/en/rest/gists/gists?apiVersion=2022-11-28#get-a-gist

        let mut resp: GetGistResponse = CustomClient::new(
            Method::GET,
            &format!("https://api.github.com/gists/{}", self.id),
        )
        .send(gh_token.as_ref())
        .await?
        .json()
        .await?;

        Ok(resp
            .files
            .remove(FILE_NAME)
            .ok_or(Error::MissingGistFiles)?
            .content)
    }
}
