use serde::Serialize;
use serde_json::json;

pub trait RestClient {
    fn builder(&mut self) -> reqwest::RequestBuilder;
    fn set_builder(&mut self, builder: reqwest::RequestBuilder);

    fn body<T: Serialize>(&mut self, serializable: T) {
        let serialized = json!(serializable).to_string();
        let builder = self.builder().body(serialized);
        self.set_builder(builder);
    }
}
