// '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;

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

impl CreateGist {
    pub fn new(description: String, public: bool, content: String) -> CreateGist {
        CreateGist {
            description,
            public,
            files: BTreeMap::from([("Installed Flatpaks".to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str) -> Result<CreateGistResponse, reqwest::Error> {
        let res = reqwest::Client::new()
            .post("https://api.github.com/gists")
            .body(json!(self).to_string())
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", github_token))
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;
        res.json().await
    }
}

impl UpdateGist {
    pub fn new(content: String) -> UpdateGist {
        UpdateGist {
            files: BTreeMap::from([("Installed Flatpaks".to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str, gist_id: &str) -> Result<(), reqwest::Error> {
        reqwest::Client::new()
            .post(&format!("https://api.github.com/gists/{}", gist_id))
            .body(json!(self).to_string())
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", github_token))
            .header("X-GitHub-Api-Version", "2022-11-28")
            .send()
            .await?;

        Ok(())
    }
}
