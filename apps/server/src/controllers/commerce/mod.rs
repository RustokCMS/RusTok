pub mod inventory;
pub mod products;
pub mod variants;

use loco_rs::controller::Routes;

pub fn routes() -> Routes {
    rustok_commerce::controllers::routes()
}
