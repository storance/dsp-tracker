mod data;
mod handler;

use actix_web::web;
pub use data::*;

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(handler::create_handler)
        .service(handler::lookup_handler)
        .service(handler::lookup_by_solar_system_handler)
        .service(handler::delete_handler)
        .service(handler::update_handler);
}
