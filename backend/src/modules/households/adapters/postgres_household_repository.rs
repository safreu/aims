use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    modules::{
        accounts::domain::UserId,
        households::{
            adapters::validate::validate_aggregate,
            domain::{
                Household, HouseholdId, HouseholdKind, HouseholdMember, HouseholdName,
                HouseholdRole,
            },
            ports::{HouseholdRepository, HouseholdRepositoryError},
        },
    },
    shared::db::map_sqlx_error,
};

pub struct PostgresHouseholdRepository {
    pool: PgPool,
}

impl PostgresHouseholdRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl HouseholdRepository for PostgresHouseholdRepository {
    async fn create_with_owner(
        &self,
        household: &Household,
        owner: &HouseholdMember,
    ) -> Result<(), HouseholdRepositoryError> {
        validate_aggregate(household, owner)?;

        let mut transaction = self.pool.begin().await.map_err(map_sqlx_error)?;

        sqlx::query!(
            r#"
            INSERT INTO households (
                id,
                name,
                kind,
                personal_owner_id,
                created_at,
                updated_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
            "#,
            household.id().into_uuid(),
            household.name().as_str(),
            household.kind().as_str(),
            household.personal_owner_id().map(|id| id.into_uuid()),
            household.created_at(),
            household.updated_at(),
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_insert_error)?;

        sqlx::query!(
            r#"
            INSERT INTO household_members (
                household_id,
                user_id,
                role,
                created_at
            )
            VALUES ($1, $2, $3, $4)
            "#,
            owner.household_id().into_uuid(),
            owner.user_id().into_uuid(),
            owner.role().as_str(),
            owner.created_at(),
        )
        .execute(&mut *transaction)
        .await
        .map_err(map_sqlx_error)?;

        transaction.commit().await.map_err(map_sqlx_error)?;

        Ok(())
    }

    async fn find_by_id(
        &self,
        id: &HouseholdId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        let row = sqlx::query_as!(
            HouseholdRow,
            r#"
            SELECT
                id,
                name,
                kind,
                personal_owner_id,
                created_at,
                updated_at
            FROM households
            WHERE id = $1
            "#,
            id.as_uuid()
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Household::try_from).transpose()
    }

    async fn find_personal_by_owner(
        &self,
        owner: &UserId,
    ) -> Result<Option<Household>, HouseholdRepositoryError> {
        let row = sqlx::query_as!(
            HouseholdRow,
            r#"
            SELECT
                id,
                name,
                kind,
                personal_owner_id,
                created_at,
                updated_at
            FROM households
            WHERE kind = 'personal' AND personal_owner_id = $1
            "#,
            owner.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Household::try_from).transpose()
    }

    async fn find_for_user(
        &self,
        user_id: &UserId,
    ) -> Result<Vec<Household>, HouseholdRepositoryError> {
        let rows = sqlx::query_as!(
            HouseholdRow,
            r#"
            SELECT
                h.id,
                h.name,
                h.kind,
                h.personal_owner_id,
                h.created_at,
                h.updated_at
            FROM households h
            JOIN household_members hm ON hm.household_id = h.id
            WHERE hm.user_id = $1
            "#,
            user_id.as_uuid(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(Household::try_from).collect()
    }

    async fn find_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<Option<HouseholdMember>, HouseholdRepositoryError> {
        let row = sqlx::query_as!(
            HouseholdMemberRow,
            r#"
            SELECT household_id, user_id, role, created_at
            FROM household_members
            WHERE household_id = $1 AND user_id = $2
            "#,
            household_id.as_uuid(),
            user_id.as_uuid(),
        )
        .fetch_optional(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        row.map(HouseholdMember::try_from).transpose()
    }

    async fn add_member(&self, member: &HouseholdMember) -> Result<(), HouseholdRepositoryError> {
        sqlx::query!(
            r#"
            INSERT INTO household_members (
                household_id,
                user_id,
                role,
                created_at
            )
            VALUES ($1, $2, $3, $4)
            "#,
            member.household_id().into_uuid(),
            member.user_id().into_uuid(),
            member.role().as_str(),
            member.created_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_add_member_error)?;

        Ok(())
    }

    async fn find_members(
        &self,
        household_id: &HouseholdId,
    ) -> Result<Vec<HouseholdMember>, HouseholdRepositoryError> {
        let rows = sqlx::query_as!(
            HouseholdMemberRow,
            r#"
            SELECT household_id, user_id, role, created_at
            FROM household_members
            WHERE household_id = $1 ORDER BY created_at
            "#,
            household_id.into_uuid()
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        rows.into_iter().map(HouseholdMember::try_from).collect()
    }

    async fn remove_member(
        &self,
        household_id: &HouseholdId,
        user_id: &UserId,
    ) -> Result<(), HouseholdRepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM household_members
            WHERE household_id = $1 AND user_id = $2
            "#,
            household_id.as_uuid(),
            user_id.as_uuid(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(HouseholdRepositoryError::MemberNotFound);
        }

        Ok(())
    }

    async fn update(&self, household: &Household) -> Result<(), HouseholdRepositoryError> {
        let result = sqlx::query!(
            r#"
            UPDATE households
            SET 
                name = $2,
                kind = $3,
                personal_owner_id = $4,
                updated_at = $5
            WHERE id = $1
            "#,
            household.id().into_uuid(),
            household.name().as_str(),
            household.kind().as_str(),
            household.personal_owner_id().map(|id| id.into_uuid()),
            household.updated_at(),
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(HouseholdRepositoryError::HouseholdNotFound);
        }

        Ok(())
    }

    async fn delete(&self, household_id: &HouseholdId) -> Result<(), HouseholdRepositoryError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM households
            WHERE id = $1
            "#,
            household_id.as_uuid()
        )
        .execute(&self.pool)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(HouseholdRepositoryError::HouseholdNotFound);
        }

        Ok(())
    }
}

const HOUSEHOLDS_PERSONAL_OWNER_UNIQUE_INDEX: &str = "households_personal_owner_unique_idx";
const HOUSEHOLDS_PRIMARY_KEY: &str = "households_pkey";

fn map_insert_error(error: sqlx::Error) -> HouseholdRepositoryError {
    if let Some(database_error) = error.as_database_error()
        && database_error.is_unique_violation()
    {
        match database_error.constraint() {
            Some(HOUSEHOLDS_PRIMARY_KEY) => {
                return HouseholdRepositoryError::HouseholdAlreadyExists;
            }
            Some(HOUSEHOLDS_PERSONAL_OWNER_UNIQUE_INDEX) => {
                return HouseholdRepositoryError::PersonalHouseholdAlreadyExists;
            }
            _ => {}
        }
    };

    map_sqlx_error(error).into()
}

const HOUSEHOLD_MEMBERS_PRIMARY_KEY: &str = "household_members_pkey";
const HOUSEHOLD_MEMBERS_HOUSEHOLD_FK: &str = "household_members_household_fk";

fn map_add_member_error(error: sqlx::Error) -> HouseholdRepositoryError {
    if let Some(database_error) = error.as_database_error() {
        match database_error.constraint() {
            Some(HOUSEHOLD_MEMBERS_PRIMARY_KEY) if database_error.is_unique_violation() => {
                return HouseholdRepositoryError::MemberAlreadyExists;
            }
            Some(HOUSEHOLD_MEMBERS_HOUSEHOLD_FK) => {
                return HouseholdRepositoryError::HouseholdNotFound;
            }
            _ => {}
        }
    };

    map_sqlx_error(error).into()
}

struct HouseholdRow {
    id: Uuid,
    name: String,
    kind: String,
    personal_owner_id: Option<Uuid>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl TryFrom<HouseholdRow> for Household {
    type Error = HouseholdRepositoryError;

    fn try_from(value: HouseholdRow) -> Result<Self, Self::Error> {
        let name = HouseholdName::parse(&value.name)
            .map_err(|_| HouseholdRepositoryError::InvalidStoredData)?;

        let kind = HouseholdKind::parse(&value.kind)
            .map_err(|_| HouseholdRepositoryError::InvalidStoredData)?;

        let personal_owner_id = value.personal_owner_id.map(UserId::from_uuid);

        Household::new(
            HouseholdId::from_uuid(value.id),
            name,
            kind,
            personal_owner_id,
            value.created_at,
            value.updated_at,
        )
        .map_err(|_| HouseholdRepositoryError::InvalidStoredData)
    }
}

struct HouseholdMemberRow {
    household_id: Uuid,
    user_id: Uuid,
    role: String,
    created_at: DateTime<Utc>,
}

impl TryFrom<HouseholdMemberRow> for HouseholdMember {
    type Error = HouseholdRepositoryError;

    fn try_from(value: HouseholdMemberRow) -> Result<Self, Self::Error> {
        let role = HouseholdRole::parse(&value.role)
            .map_err(|_| HouseholdRepositoryError::InvalidStoredData)?;

        Ok(HouseholdMember::new(
            HouseholdId::from_uuid(value.household_id),
            UserId::from_uuid(value.user_id),
            role,
            value.created_at,
        ))
    }
}
