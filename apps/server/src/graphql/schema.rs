use async_graphql::{EmptySubscription, MergedObject, Schema};
use sea_orm::DatabaseConnection;

use rustok_core::EventBus;

use super::alloy::{AlloyMutation, AlloyQuery, AlloyState};
use super::blog::{BlogMutation, BlogQuery};
use super::commerce::{CommerceMutation, CommerceQuery};
use super::content::{ContentMutation, ContentQuery};
use super::forum::{ForumMutation, ForumQuery};
use super::mutations::RootMutation;
use super::queries::RootQuery;

#[derive(MergedObject, Default)]
pub struct Query(
    RootQuery,
    CommerceQuery,
    ContentQuery,
    BlogQuery,
    ForumQuery,
    AlloyQuery,
);

#[derive(MergedObject, Default)]
pub struct Mutation(
    RootMutation,
    CommerceMutation,
    ContentMutation,
    BlogMutation,
    ForumMutation,
    AlloyMutation,
);

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(
    db: DatabaseConnection,
    event_bus: EventBus,
    alloy_state: AlloyState,
) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .data(event_bus)
        .data(alloy_state)
        .finish()
}
