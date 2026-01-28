pub mod tenant_modules;
pub mod tenants;
pub mod users;

pub mod prelude {
    pub use super::tenant_modules::Entity as TenantModules;
    pub use super::tenants::Entity as Tenants;
    pub use super::users::Entity as Users;
}
