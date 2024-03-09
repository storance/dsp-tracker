use crate::{
    error::Result,
    star::{
        api::{CreateStarRequest, Star, UpdateStarRequest},
        domain,
    },
    AppState,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use log::error;
use uuid::Uuid;

#[post("/solar-systems/{solarSystemId}/star")]
async fn create_handler(
    path: web::Path<Uuid>,
    request: web::Json<CreateStarRequest>,
    data: web::Data<AppState>,
) -> Result<Star> {
    let mut transaction = data.db.begin().await?;
    let solar_system_id = path.into_inner();

    let solar_system = domain::Star::new(
        solar_system_id,
        request.spectral_class,
        request.luminosity,
        request.radius,
    );

    let response = domain::create(&mut transaction, &solar_system)
        .await
        .inspect_err(|err| error!("Failed to create star: {}", err))?;
    transaction.commit().await?;

    Ok(response.into())
}

#[get("/stars/{id}")]
async fn lookup_handler(path: web::Path<Uuid>, data: web::Data<AppState>) -> Result<Star> {
    let mut transaction = data.db.begin().await?;

    let id = path.into_inner();
    let response = domain::lookup(&mut transaction, id)
        .await
        .inspect_err(|err| error!("Failed to lookup solar system with id `{}`: {}", id, err))
        .map(Star::from)?;

    transaction.commit().await?;
    Ok(response)
}

#[get("/solar-systems/{solarSystemId}/star")]
async fn lookup_by_solar_system_handler(
    path: web::Path<Uuid>,
    data: web::Data<AppState>,
) -> Result<Star> {
    let mut transaction = data.db.begin().await?;

    let solar_system_id = path.into_inner();
    let response = domain::lookup_by_solar_system_id(&mut transaction, solar_system_id)
        .await
        .inspect_err(|err| {
            error!(
                "Failed to lookup star with solar system id `{}`: {}",
                solar_system_id, err
            )
        })
        .map(Star::from)?;

    transaction.commit().await?;
    Ok(response)
}

#[delete("/stars/{id}")]
async fn delete_handler(path: web::Path<Uuid>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let mut transaction = data.db.begin().await?;
    let id = path.into_inner();

    domain::delete(&mut transaction, id).await?;
    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

#[patch("/stars/{id}")]
async fn update_handler(
    path: web::Path<Uuid>,
    request: web::Json<UpdateStarRequest>,
    data: web::Data<AppState>,
) -> Result<Star> {
    let mut transaction = data.db.begin().await?;
    let id = path.into_inner();

    let mut star = domain::lookup(&mut transaction, id).await?;
    if let Some(spectral_class) = request.spectral_class {
        star.spectral_class = spectral_class;
    }

    if let Some(luminosity) = request.luminosity {
        star.luminosity = luminosity;
    }

    if let Some(radius) = request.radius {
        star.radius = radius;
    }

    let response = domain::update(&mut transaction, &star)
        .await
        .inspect_err(|err| error!("Failed to update star with id `{}`: {}", id, err))?;

    transaction.commit().await?;
    Ok(response.into())
}
