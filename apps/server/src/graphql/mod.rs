pub mod common;
pub mod errors;
pub mod mutations;
pub mod queries;
pub mod types;

use async_graphql::{EmptySubscription, Schema};
use mutations::RootMutation;
use queries::RootQuery;

pub type AppSchema = Schema<RootQuery, RootMutation, EmptySubscription>;

pub fn build_schema() -> AppSchema {
    Schema::build(RootQuery::default(), RootMutation::default(), EmptySubscription).finish()
}
