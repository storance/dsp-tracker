use super::{CreateSolarSystemRequest, SolarSystem, UpdateSolarSystemRequest};
use crate::solar_system::api::{SearchRequest, SearchRequestRaw};
use crate::solar_system::domain;
use crate::{data::Page, error::Result, AppState};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use log::error;
use uuid::Uuid;

#[post("/saves/{saveId}/solar-systems")]
async fn create_handler(
    path: web::Path<Uuid>,
    request: web::Json<CreateSolarSystemRequest>,
    data: web::Data<AppState>,
) -> Result<SolarSystem> {
    let mut transaction = data.db.begin().await?;
    let save_id = path.into_inner();

    let solar_system =
        domain::SolarSystem::new(save_id, request.name.clone(), request.notes.clone());

    let response = domain::create(&mut transaction, &solar_system)
        .await
        .inspect_err(|err| error!("Failed to create solar system {}: {}", request.name, err))?;
    transaction.commit().await?;

    Ok(response.into())
}

#[get("/solar-systems/{id}")]
async fn lookup_handler(path: web::Path<Uuid>, data: web::Data<AppState>) -> Result<SolarSystem> {
    let mut transaction = data.db.begin().await?;

    let id = path.into_inner();
    let response = domain::lookup(&mut transaction, id)
        .await
        .inspect_err(|err| error!("Failed to lookup solar system with id `{}`: {}", id, err))
        .map(SolarSystem::from)?;

    transaction.commit().await?;
    Ok(response)
}

#[delete("/solar-systems/{id}")]
async fn delete_handler(path: web::Path<Uuid>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let mut transaction = data.db.begin().await?;
    let id = path.into_inner();

    domain::delete(&mut transaction, id).await?;
    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/saves/{saveId}/solar-systems")]
async fn search_handler(
    path: web::Path<Uuid>,
    query: web::Query<SearchRequestRaw>,
    data: web::Data<AppState>,
) -> Result<Page<SolarSystem>> {
    let mut transaction = data.db.begin().await?;
    let save_id = path.into_inner();
    let search_params = SearchRequest::try_from(query.into_inner())?;

    let response = domain::search(&mut transaction, save_id, &search_params)
        .await
        .map(|r| r.map(|s| SolarSystem::from(s)))
        .inspect_err(|err| error!("Failed to search for solar systems: {}", err))?;
    transaction.commit().await?;
    Ok(response)
}

#[patch("/solar-systems/{id}")]
async fn update_handler(
    path: web::Path<Uuid>,
    request: web::Json<UpdateSolarSystemRequest>,
    data: web::Data<AppState>,
) -> Result<SolarSystem> {
    let mut transaction = data.db.begin().await?;
    let id = path.into_inner();

    let mut solar_system = domain::lookup(&mut transaction, id).await?;
    if let Some(name) = &request.name {
        solar_system.name = name.clone();
    }

    if let Some(notes) = &request.notes {
        solar_system.notes = notes.clone();
    }

    let response = domain::update(&mut transaction, &solar_system)
        .await
        .inspect_err(|err| error!("Failed to update save with id `{}`: {}", id, err))?;

    transaction.commit().await?;
    Ok(response.into())
}
