mod m20260328_000001_create_blog_post_tables;
mod m20260328_000002_create_blog_taxonomy_tables;

use sea_orm_migration::MigrationTrait;

pub fn migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(m20260328_000001_create_blog_post_tables::Migration),
        Box::new(m20260328_000002_create_blog_taxonomy_tables::Migration),
    ]
}
