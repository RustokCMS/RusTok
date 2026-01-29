use async_graphql::{EmptySubscription, MergedObject, Schema};
use sea_orm::DatabaseConnection;

use rustok_core::EventBus;

use super::commerce::{CommerceMutation, CommerceQuery};
use super::mutations::RootMutation;
use super::queries::RootQuery;

#[derive(MergedObject, Default)]
pub struct Query(RootQuery, CommerceQuery);

#[derive(MergedObject, Default)]
pub struct Mutation(RootMutation, CommerceMutation);

pub type AppSchema = Schema<Query, Mutation, EmptySubscription>;

pub fn build_schema(db: DatabaseConnection, event_bus: EventBus) -> AppSchema {
    Schema::build(Query::default(), Mutation::default(), EmptySubscription)
        .data(db)
        .data(event_bus)
        .finish()
}
