use super::{CreateGameSaveRequest, GameSave, UpdateGameSaveRequest};
use crate::{
    data::{Page, PageRequest, PageRequestRaw},
    error::Result,
    game_save::domain,
    AppState,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use log::error;
use uuid::Uuid;

#[post("/saves")]
async fn create_handler(
    request: web::Json<CreateGameSaveRequest>,
    data: web::Data<AppState>,
) -> Result<GameSave> {
    let mut transaction = data.db.begin().await?;

    let save = domain::GameSave::new(
        request.name.clone(),
        request.notes.clone(),
        request.mining_speed,
    );
    let response = domain::create(&mut transaction, &save)
        .await
        .inspect_err(|err| error!("Failed to create save {}: {}", save.name, err))?;

    transaction.commit().await?;
    Ok(response.into())
}

#[get("/saves/{id}")]
async fn lookup_handler(path: web::Path<Uuid>, data: web::Data<AppState>) -> Result<GameSave> {
    let mut transaction = data.db.begin().await?;

    let id = path.into_inner();
    let response = domain::lookup(&mut transaction, id)
        .await
        .inspect_err(|err| error!("Failed to lookup save with id `{}`: {}", id, err))?;

    transaction.commit().await?;
    Ok(response.into())
}

#[delete("/saves/{id}")]
async fn delete_handler(path: web::Path<Uuid>, data: web::Data<AppState>) -> Result<HttpResponse> {
    let mut transaction = data.db.begin().await?;
    let id = path.into_inner();

    domain::delete(&mut transaction, id).await?;
    transaction.commit().await?;

    Ok(HttpResponse::NoContent().finish())
}

#[get("/saves")]
async fn search_handler(
    query: web::Query<PageRequestRaw>,
    data: web::Data<AppState>,
) -> Result<Page<GameSave>> {
    let mut transaction = data.db.begin().await?;
    let page_params = PageRequest::try_from(query.into_inner())?;

    let response = domain::search(&mut transaction, &page_params)
        .await
        .map(Page::convert)
        .inspect_err(|err| error!("Failed to search for saves: {}", err))?;
    transaction.commit().await?;
    Ok(response)
}

#[patch("/saves/{id}")]
async fn update_handler(
    path: web::Path<Uuid>,
    request: web::Json<UpdateGameSaveRequest>,
    data: web::Data<AppState>,
) -> Result<GameSave> {
    let mut transaction = data.db.begin().await?;
    let id = path.into_inner();

    let mut save = domain::lookup(&mut transaction, id).await?;
    if let Some(name) = &request.name {
        save.name = name.clone();
    }

    if let Some(mining_speed) = request.mining_speed {
        save.mining_speed = mining_speed;
    }

    let response = domain::update(&mut transaction, &save)
        .await
        .map(GameSave::from)
        .inspect_err(|err| error!("Failed to update save with id `{}`: {}", id, err))?;

    transaction.commit().await?;
    Ok(response)
}
