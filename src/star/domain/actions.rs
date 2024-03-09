use super::{Star, StarColumns};
use crate::{
    error::{ObjectKind, Result, TrackerError},
    field::FieldValue,
    solar_system::SolarSystemColumns,
};
use sea_query::{Alias, Asterisk, Expr, PostgresQueryBuilder, Query};
use sea_query_binder::SqlxBinder;
use sqlx::{error::ErrorKind, Postgres, Transaction};
use uuid::Uuid;

pub async fn create<'a>(tx: &mut Transaction<'a, Postgres>, star: &Star) -> Result<Star> {
    let (sql, values) = Query::insert()
        .into_table(StarColumns::Table)
        .columns([
            StarColumns::Id,
            StarColumns::CreatedAt,
            StarColumns::Version,
            StarColumns::SolarSystemId,
            StarColumns::SpectralClass,
            StarColumns::Luminosity,
            StarColumns::Radius,
        ])
        .values_panic([
            star.id.into(),
            Expr::current_timestamp().into(),
            star.version.into(),
            star.solar_system_id.into(),
            Expr::val(star.spectral_class.as_ref())
                .as_enum(Alias::new("spectral_class"))
                .into(),
            star.luminosity.into(),
            star.radius.into(),
        ])
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await
        .map_err(|err| map_constraint_errors(err, star))?;

    lookup(tx, star.id)
        .await
        .map_err(TrackerError::not_found_unexpected)
}

pub async fn update<'a>(tx: &mut Transaction<'a, Postgres>, star: &Star) -> Result<Star> {
    let (sql, values) = Query::update()
        .table(StarColumns::Table)
        .values([
            (StarColumns::UpdatedAt, Expr::current_timestamp().into()),
            (StarColumns::Version, Expr::col(StarColumns::Version).add(1)),
            (
                StarColumns::SpectralClass,
                Expr::val(star.spectral_class.as_ref())
                    .as_enum(Alias::new("spectral_class"))
                    .into(),
            ),
            (StarColumns::Luminosity, star.luminosity.into()),
            (StarColumns::Radius, star.radius.into()),
        ])
        .and_where(Expr::col(StarColumns::Id).eq(star.id))
        .and_where(Expr::col(StarColumns::Version).eq(star.version))
        .build_sqlx(PostgresQueryBuilder);

    let rows_updated = sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await
        .map_err(|err| map_constraint_errors(err, star))?
        .rows_affected();

    if rows_updated == 0 {
        Err(TrackerError::concurrent_update(
            ObjectKind::Star,
            FieldValue::new(StarColumns::Id, star.id),
        ))
    } else {
        lookup(tx, star.id).await
    }
}

pub async fn lookup_optional<'a>(
    tx: &mut Transaction<'a, Postgres>,
    id: Uuid,
) -> Result<Option<Star>> {
    let (sql, values) = Query::select()
        .column((Alias::new("solar_system"), Asterisk))
        .from_as(StarColumns::Table, Alias::new("solar_system"))
        .and_where(Expr::col(StarColumns::Id).eq(id))
        .limit(1)
        .build_sqlx(PostgresQueryBuilder);

    Ok(sqlx::query_as_with::<_, Star, _>(&sql, values.clone())
        .fetch_optional(&mut **tx)
        .await?)
}

pub async fn lookup<'a>(tx: &mut Transaction<'a, Postgres>, id: Uuid) -> Result<Star> {
    lookup_optional(tx, id)
        .await
        .transpose()
        .unwrap_or_else(|| {
            Err(TrackerError::not_found(
                ObjectKind::Star,
                FieldValue::new(StarColumns::Id, id),
            ))
        })
}

pub async fn lookup_by_solar_system_id<'a>(
    tx: &mut Transaction<'a, Postgres>,
    solar_system_id: Uuid,
) -> Result<Star> {
    let (sql, values) = Query::select()
        .column((Alias::new("solar_system"), Asterisk))
        .from_as(StarColumns::Table, Alias::new("solar_system"))
        .and_where(Expr::col(StarColumns::SolarSystemId).eq(solar_system_id))
        .limit(1)
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_as_with::<_, Star, _>(&sql, values.clone())
        .fetch_optional(&mut **tx)
        .await?
        .map(Ok)
        .unwrap_or_else(|| {
            Err(TrackerError::not_found(
                ObjectKind::Star,
                FieldValue::new(StarColumns::SolarSystemId, solar_system_id),
            ))
        })
}

pub async fn delete<'a>(tx: &mut Transaction<'a, Postgres>, id: Uuid) -> Result<()> {
    let (sql, values) = Query::delete()
        .from_table(StarColumns::Table)
        .and_where(Expr::col(StarColumns::Id).eq(id))
        .build_sqlx(PostgresQueryBuilder);

    sqlx::query_with(&sql, values.clone())
        .execute(&mut **tx)
        .await?;
    Ok(())
}

fn map_constraint_errors(err: sqlx::Error, star: &Star) -> TrackerError {
    match &err {
        sqlx::Error::Database(db_err) => match (db_err.kind(), db_err.constraint()) {
            (ErrorKind::UniqueViolation, Some("stars_solar_system_id_key")) => {
                TrackerError::duplicate(
                    ObjectKind::Star,
                    [FieldValue::new(
                        StarColumns::SolarSystemId,
                        star.solar_system_id,
                    )],
                )
            }
            (ErrorKind::ForeignKeyViolation, Some("stars_solar_system_id_fkey")) => {
                TrackerError::not_found(
                    ObjectKind::SolarSystem,
                    FieldValue::new(SolarSystemColumns::Id, star.solar_system_id),
                )
            }
            _ => TrackerError::from(err),
        },
        _ => TrackerError::from(err),
    }
}
