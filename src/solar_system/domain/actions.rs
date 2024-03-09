use super::{SolarSystem, SolarSystemColumns};
use crate::{
    data::{Page, PageMetadata, Sort},
    error::{ObjectKind, Result, TrackerError},
    field::{Field, FieldValue},
    game_save::GameSaveColumns,
    solar_system::api::{SearchRequest, SolarSystemFields},
};
use sea_query::{
    extension::postgres::PgBinOper, Alias, Asterisk, Expr, Func, Iden, PostgresQueryBuilder, Query,
    SelectStatement,
};
use sea_query_binder::SqlxBinder;
use sqlx::{error::ErrorKind, Postgres, Row, Transaction};
use uuid::Uuid;

pub async fn create<'a>(
    tx: &mut Transaction<'a, Postgres>,
    solar_system: &SolarSystem,
) -> Result<SolarSystem> {
    let (sql, values) = Query::insert()
        .into_table(SolarSystemColumns::Table)
        .columns([
            SolarSystemColumns::Id,
            SolarSystemColumns::CreatedAt,
            SolarSystemColumns::Version,
            SolarSystemColumns::SaveId,
            SolarSystemColumns::Name,
            SolarSystemColumns::Notes,
        ])
        .values_panic([
            solar_system.id.into(),
            Expr::current_timestamp().into(),
            solar_system.version.into(),
            solar_system.save_id.into(),
            (&solar_system.name).into(),
            solar_system.notes.as_deref().into(),
        ])
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await
        .map_err(|err| map_constraint_errors(err, solar_system))?;

    lookup(tx, solar_system.id)
        .await
        .map_err(TrackerError::not_found_unexpected)
}

pub async fn update<'a>(
    tx: &mut Transaction<'a, Postgres>,
    solar_system: &SolarSystem,
) -> Result<SolarSystem> {
    let (sql, values) = Query::update()
        .table(SolarSystemColumns::Table)
        .values([
            (
                SolarSystemColumns::UpdatedAt,
                Expr::current_timestamp().into(),
            ),
            (
                SolarSystemColumns::Version,
                Expr::col(SolarSystemColumns::Version).add(1),
            ),
            (SolarSystemColumns::Name, solar_system.name.clone().into()),
            (SolarSystemColumns::Notes, solar_system.notes.clone().into()),
        ])
        .and_where(Expr::col(SolarSystemColumns::Id).eq(solar_system.id))
        .and_where(Expr::col(SolarSystemColumns::Version).eq(solar_system.version))
        .build_sqlx(PostgresQueryBuilder);

    let rows_updated = sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await
        .map_err(|err| map_constraint_errors(err, solar_system))?
        .rows_affected();

    if rows_updated == 0 {
        Err(TrackerError::concurrent_update(
            ObjectKind::SolarSystem,
            FieldValue::new(SolarSystemColumns::Id, solar_system.id),
        ))
    } else {
        lookup(tx, solar_system.id).await
    }
}

pub async fn lookup_optional<'a>(
    tx: &mut Transaction<'a, Postgres>,
    id: Uuid,
) -> Result<Option<SolarSystem>> {
    let (sql, values) = Query::select()
        .column((Alias::new("solar_system"), Asterisk))
        .from_as(SolarSystemColumns::Table, Alias::new("solar_system"))
        .and_where(Expr::col(SolarSystemColumns::Id).eq(id))
        .limit(1)
        .build_sqlx(PostgresQueryBuilder);

    Ok(
        sqlx::query_as_with::<_, SolarSystem, _>(&sql, values.clone())
            .fetch_optional(&mut **tx)
            .await?,
    )
}

pub async fn lookup<'a>(tx: &mut Transaction<'a, Postgres>, id: Uuid) -> Result<SolarSystem> {
    lookup_optional(tx, id)
        .await
        .transpose()
        .unwrap_or_else(|| {
            Err(TrackerError::not_found(
                ObjectKind::SolarSystem,
                FieldValue::new(SolarSystemColumns::Id, id),
            ))
        })
}

pub async fn search<'a>(
    tx: &mut Transaction<'a, Postgres>,
    save_id: Uuid,
    search_params: &SearchRequest,
) -> Result<Page<SolarSystem>> {
    let page_req = &search_params.page_request;
    let mut joins_tracker = Vec::new();

    let mut select_count_stmt = Query::select()
        .expr(Func::count(Expr::col(Asterisk)))
        .from(SolarSystemColumns::Table)
        .to_owned();
    add_where_clause(&mut select_count_stmt, save_id, search_params);

    let (count_sql, count_values) = select_count_stmt.build_sqlx(PostgresQueryBuilder);

    let total_results: i64 = sqlx::query_with(&count_sql, count_values.clone())
        .fetch_one(&mut **tx)
        .await?
        .get(0);

    let mut select_stmt = Query::select()
        .expr(Expr::col(Asterisk))
        .from(SolarSystemColumns::Table)
        .limit(page_req.size)
        .offset(page_req.offset())
        .to_owned();
    add_where_clause(&mut select_stmt, save_id, search_params);
    add_sorts(&mut select_stmt, &page_req.sorts, &mut joins_tracker);

    let (sql, values) = select_stmt.build_sqlx(PostgresQueryBuilder);

    Ok(
        sqlx::query_as_with::<_, SolarSystem, _>(&sql, values.clone())
            .fetch_all(&mut **tx)
            .await
            .map(|result| {
                Page::new(
                    result,
                    PageMetadata::new(page_req.page, page_req.size, total_results as u64),
                )
            })?,
    )
}

pub async fn delete<'a>(tx: &mut Transaction<'a, Postgres>, id: Uuid) -> Result<()> {
    let (sql, values) = Query::delete()
        .from_table(SolarSystemColumns::Table)
        .and_where(Expr::col(SolarSystemColumns::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn add_where_clause(select_stmt: &mut SelectStatement, save_id: Uuid, req: &SearchRequest) {
    select_stmt.and_where(Expr::col(SolarSystemColumns::SaveId).eq(save_id));

    if let Some(name) = &req.name {
        let pattern = format!("(^|\\s+){0}", regex::escape(name));
        select_stmt.and_where(
            Expr::col(SolarSystemColumns::Name).binary(PgBinOper::RegexCaseInsensitive, pattern),
        );
    }
}

fn add_sorts(
    select_stmt: &mut SelectStatement,
    sorts: &[Sort<SolarSystemFields>],
    joins_tracker: &mut Vec<String>,
) {
    for sort in sorts {
        add_join_for_field(select_stmt, sort.field, joins_tracker);
        select_stmt.order_by(sort.field.column(), sort.direction.into());
    }
}

pub fn add_join_for_field(
    select_stmt: &mut SelectStatement,
    field: SolarSystemFields,
    joins_tracker: &mut Vec<String>,
) {
    if let SolarSystemFields::Save(..) = field {
        let save_table = GameSaveColumns::Table.to_string();
        if !joins_tracker.contains(&save_table) {
            joins_tracker.push(save_table);
            select_stmt.left_join(
                GameSaveColumns::Table,
                Expr::col((GameSaveColumns::Table, GameSaveColumns::Id))
                    .equals((SolarSystemColumns::Table, SolarSystemColumns::SaveId)),
            );
        }
    }
}

fn map_constraint_errors(err: sqlx::Error, solar_system: &SolarSystem) -> TrackerError {
    match &err {
        sqlx::Error::Database(db_err) => match (db_err.kind(), db_err.constraint()) {
            (ErrorKind::UniqueViolation, Some("solar_systems_save_id_name_key")) => {
                TrackerError::duplicate(
                    ObjectKind::SolarSystem,
                    [
                        FieldValue::new(SolarSystemColumns::SaveId, solar_system.save_id),
                        FieldValue::new(SolarSystemColumns::Name, &solar_system.name),
                    ],
                )
            }
            (ErrorKind::ForeignKeyViolation, Some("solar_systems_save_id_fkey")) => {
                TrackerError::not_found(
                    ObjectKind::Save,
                    FieldValue::new(GameSaveColumns::Id, solar_system.save_id),
                )
            }
            _ => TrackerError::from(err),
        },
        _ => TrackerError::from(err),
    }
}
