use async_trait::async_trait;
use rustok_core::RusToKModule;

pub struct BlogModule;

#[async_trait]
impl RusToKModule for BlogModule {
    fn slug(&self) -> &'static str {
        "blog"
    }

    fn name(&self) -> &'static str {
        "Blog"
    }

    fn description(&self) -> &'static str {
        "Posts, Pages, Comments"
    }

    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}
