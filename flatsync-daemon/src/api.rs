use const_format::formatcp;
use libflatsync_common::FlatpakInstallationMap;
use reqwest::header;
// '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

const USER_AGENT: &str = formatcp!("flatsync-daemon / {}", libflatsync_common::config::VERSION);
const FILE_NAME: &str = "flatsync.json";

#[derive(Serialize, Deserialize)]
struct GistFile {
    content: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGist {
    description: String,
    public: bool,
    files: BTreeMap<String, GistFile>,
}

#[derive(Serialize, Deserialize)]
pub struct CreateGistResponse {
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct UpdateGist {
    files: BTreeMap<String, GistFile>,
}

#[derive(Debug)]
pub struct GetGist {
    id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetGistResponse {
    files: BTreeMap<String, GistFile>,
}

impl CreateGist {
    pub fn new(description: String, public: bool, content: String) -> CreateGist {
        CreateGist {
            description,
            public,
            files: BTreeMap::from([(FILE_NAME.to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str) -> Result<CreateGistResponse, reqwest::Error> {
        let res = reqwest::Client::new()
            .post("https://api.github.com/gists")
            .body(json!(self).to_string())
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::AUTHORIZATION, format!("Bearer {}", github_token))
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;
        res.json().await
    }
}

impl UpdateGist {
    pub fn new(content: String) -> UpdateGist {
        UpdateGist {
            files: BTreeMap::from([(FILE_NAME.to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str, gist_id: &str) -> Result<(), reqwest::Error> {
        reqwest::Client::new()
            .post(&format!("https://api.github.com/gists/{}", gist_id))
            .body(json!(self).to_string())
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(header::AUTHORIZATION, format!("Bearer {}", github_token))
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        Ok(())
    }
}

impl GetGist {
    pub fn new<S: AsRef<str>>(id: S) -> Self {
        Self {
            id: id.as_ref().into(),
        }
    }

    pub async fn get<S: AsRef<str>>(
        &self,
        gh_token: S,
    ) -> Result<FlatpakInstallationMap, reqwest::Error> {
        // See https://docs.github.com/en/rest/gists/gists?apiVersion=2022-11-28#get-a-gist

        let resp: GetGistResponse = reqwest::Client::new()
            .get(&format!("https://api.github.com/gists/{}", self.id))
            .header(header::USER_AGENT, USER_AGENT)
            .header(header::ACCEPT, "application/vnd.github+json")
            .header(
                header::AUTHORIZATION,
                format!("Bearer {}", gh_token.as_ref()),
            )
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?
            .json()
            .await?;

        Ok(serde_json::from_str::<FlatpakInstallationMap>(
            &resp.files.get(FILE_NAME).unwrap().content,
        )
        .unwrap())
    }
}
