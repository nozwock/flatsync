pub mod data_sink;
pub mod data_sink_client;
pub mod github;
mod oauth_client;
mod rest_client;

pub use github::GitHubGistDataSink;
