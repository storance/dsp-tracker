use super::data::{GameSave, GameSaveColumns};
use crate::data::{Page, PageMetadata, PageRequest, Sort};
use crate::error::{ObjectKind, Result, TrackerError};
use crate::field::{Field, FieldValue};
use crate::game_save::api::SaveFields;
use sea_query::{Asterisk, Expr, Func, PostgresQueryBuilder, Query, SelectStatement};
use sea_query_binder::SqlxBinder;
use sqlx::{Postgres, Row, Transaction};
use uuid::Uuid;

pub async fn create<'a>(tx: &mut Transaction<'a, Postgres>, save: &GameSave) -> Result<GameSave> {
    let (sql, values) = Query::insert()
        .into_table(GameSaveColumns::Table)
        .columns([
            GameSaveColumns::Id,
            GameSaveColumns::CreatedAt,
            GameSaveColumns::Version,
            GameSaveColumns::Name,
            GameSaveColumns::MiningSpeed,
        ])
        .values_panic([
            save.id.into(),
            Expr::current_timestamp().into(),
            save.version.into(),
            (&save.name).into(),
            save.mining_speed.into(),
        ])
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await
        .map_err(|err| map_constraint_errors(err, save))?;

    lookup(tx, save.id)
        .await
        .map_err(TrackerError::not_found_unexpected)
}

pub async fn update<'a>(tx: &mut Transaction<'a, Postgres>, save: &GameSave) -> Result<GameSave> {
    let (sql, values) = Query::update()
        .table(GameSaveColumns::Table)
        .values([
            (GameSaveColumns::UpdatedAt, Expr::current_timestamp().into()),
            (
                GameSaveColumns::Version,
                Expr::col(GameSaveColumns::Version).add(1),
            ),
            (GameSaveColumns::Name, save.name.clone().into()),
            (GameSaveColumns::MiningSpeed, save.mining_speed.into()),
        ])
        .and_where(Expr::col(GameSaveColumns::Id).eq(save.id))
        .and_where(Expr::col(GameSaveColumns::Version).eq(save.version))
        .build_sqlx(PostgresQueryBuilder);

    let rows_updated = sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await
        .map_err(|err| map_constraint_errors(err, save))?
        .rows_affected();

    if rows_updated == 0 {
        Err(TrackerError::concurrent_update(
            ObjectKind::Save,
            FieldValue::new(GameSaveColumns::Id, save.id),
        ))
    } else {
        lookup(tx, save.id).await
    }
}

pub async fn lookup_optional<'a>(
    tx: &mut Transaction<'a, Postgres>,
    id: Uuid,
) -> Result<Option<GameSave>> {
    let (sql, values) = Query::select()
        .expr(Expr::col(Asterisk))
        .from(GameSaveColumns::Table)
        .and_where(Expr::col(GameSaveColumns::Id).eq(id))
        .limit(1)
        .build_sqlx(PostgresQueryBuilder);

    Ok(sqlx::query_as_with::<_, GameSave, _>(&sql, values.clone())
        .fetch_optional(&mut **tx)
        .await?)
}

pub async fn lookup<'a>(tx: &mut Transaction<'a, Postgres>, id: Uuid) -> Result<GameSave> {
    lookup_optional(tx, id)
        .await
        .transpose()
        .unwrap_or_else(|| {
            Err(TrackerError::not_found(
                ObjectKind::Save,
                FieldValue::new(GameSaveColumns::Id, id),
            ))
        })
}

pub async fn search<'a>(
    tx: &mut Transaction<'a, Postgres>,
    page_params: &PageRequest<SaveFields>,
) -> Result<Page<GameSave>> {
    let (count_sql, count_values) = Query::select()
        .expr(Func::count(Expr::col(Asterisk)))
        .from(GameSaveColumns::Table)
        .build_sqlx(PostgresQueryBuilder);

    let total_results: i64 = sqlx::query_with(&count_sql, count_values.clone())
        .fetch_one(&mut **tx)
        .await?
        .get(0);

    let mut select_stmt = Query::select()
        .expr(Expr::col(Asterisk))
        .from(GameSaveColumns::Table)
        .limit(page_params.size)
        .offset(page_params.offset())
        .to_owned();
    add_sorts(&mut select_stmt, &page_params.sorts);

    let (sql, values) = select_stmt.build_sqlx(PostgresQueryBuilder);

    Ok(sqlx::query_as_with::<_, GameSave, _>(&sql, values.clone())
        .fetch_all(&mut **tx)
        .await
        .map(|result| {
            Page::new(
                result,
                PageMetadata::new(page_params.page, page_params.size, total_results as u64),
            )
        })?)
}

pub async fn delete<'a>(tx: &mut Transaction<'a, Postgres>, id: Uuid) -> Result<()> {
    let (sql, values) = Query::delete()
        .from_table(GameSaveColumns::Table)
        .and_where(Expr::col(GameSaveColumns::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn add_sorts(select_stmt: &mut SelectStatement, sorts: &[Sort<SaveFields>]) {
    for sort in sorts {
        select_stmt.order_by(sort.field.column(), sort.direction.into());
    }
}

fn map_constraint_errors(err: sqlx::Error, save: &GameSave) -> TrackerError {
    match &err {
        sqlx::Error::Database(db_err) => {
            if db_err.is_unique_violation() {
                match db_err.constraint() {
                    Some("saves_name_key") => TrackerError::duplicate(
                        ObjectKind::Save,
                        FieldValue::new(GameSaveColumns::Name, &save.name),
                    ),
                    Some("saves_id_pkey") => TrackerError::duplicate(
                        ObjectKind::Save,
                        FieldValue::new(GameSaveColumns::Id, save.id),
                    ),
                    _ => TrackerError::from(err),
                }
            } else {
                TrackerError::from(err)
            }
        }
        _ => TrackerError::from(err),
    }
}
