use libflatsync_common::FlatpakInstallationMap;
// '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
use crate::Error;
use reqwest::{IntoUrl, Method, RequestBuilder};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::json;
use std::collections::BTreeMap;
use zbus::zvariant::Type;

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

// GitHub-Gists expects us to send the content of the file as a string, not as a JSON object.
fn content_to_string<S>(f: &FlatpakInstallationMap, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&serde_json::to_string_pretty(&f).unwrap())
}

fn string_to_content<'de, D>(deserializer: D) -> Result<FlatpakInstallationMap, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    serde_json::from_str(&s).map_err(serde::de::Error::custom)
}

#[derive(Serialize, Deserialize)]
struct GistFile {
    #[serde(serialize_with = "content_to_string")]
    #[serde(deserialize_with = "string_to_content")]
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

#[derive(Serialize, Deserialize, Type)]
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
