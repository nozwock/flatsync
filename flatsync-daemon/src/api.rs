// '{"description":"Example of a gist","public":false,"files":{"README.md":{"content":"Hello World"}}}'
use serde::{Deserialize, Serialize};
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

    pub async fn post(&self, github_token: &str) -> Result<CreateGistResponse, surf::Error> {
        let mut res = surf::post("https://api.github.com/gists")
            .body_json(self)?
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", github_token))
            .header("X-GitHub-Api-Version", "2022-11-28")
            .await?;
        res.body_json().await
    }
}

impl UpdateGist {
    pub fn new(content: String) -> UpdateGist {
        UpdateGist {
            files: BTreeMap::from([("Installed Flatpaks".to_string(), GistFile { content })]),
        }
    }

    pub async fn post(&self, github_token: &str, gist_id: &str) -> Result<(), surf::Error> {
        surf::post(&format!("https://api.github.com/gists/{}", gist_id))
            .body_json(self)?
            .header("Accept", "application/vnd.github+json")
            .header("Authorization", format!("Bearer {}", github_token))
            .header("X-GitHub-Api-Version", "2022-11-28")
            .await?;

        Ok(())
    }
}
