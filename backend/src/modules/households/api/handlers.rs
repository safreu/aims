use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use uuid::Uuid;

use crate::{
    modules::{
        accounts::{api::CurrentUser, domain::UserId},
        households::{
            api::dto::{
                AddHouseholdMemberRequest, CreateHouseholdRequest, CreateHouseholdResponse,
                HouseholdMemberResponse, HouseholdResponse, RenameHouseholdRequest,
            },
            application::{
                AddHouseholdMemberCommand, CreateHouseholdCommand, DeleteHouseholdCommand,
                GetHouseholdCommand, LeaveHouseholdCommand, ListHouseholdMembersCommand,
                ListHouseholdsForUserCommand, RemoveHouseholdMemberCommand, RenameHouseholdCommand,
            },
            domain::{HouseholdId, HouseholdKind},
        },
    },
    shared::api::{ApiError, AppState},
};

pub async fn create_household(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(request): Json<CreateHouseholdRequest>,
) -> Result<(StatusCode, Json<CreateHouseholdResponse>), ApiError> {
    let kind = HouseholdKind::parse(&request.kind).map_err(|_| {
        ApiError::bad_request("invalid_household_kind", "The household kind is invalid")
    })?;

    let command = CreateHouseholdCommand {
        owner_id: current_user.user_id(),
        name: request.name,
        kind,
    };

    let household_id = state
        .households
        .create_household
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok((
        StatusCode::CREATED,
        Json(CreateHouseholdResponse {
            id: household_id.to_string(),
        }),
    ))
}

pub async fn list_households(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> Result<Json<Vec<HouseholdResponse>>, ApiError> {
    let command = ListHouseholdsForUserCommand {
        user_id: current_user.user_id(),
    };

    let households = state
        .households
        .list_households_for_user
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let response = households
        .into_iter()
        .map(|household| HouseholdResponse {
            id: household.id().to_string(),
            name: household.name().as_str().to_owned(),
            kind: household.kind().to_string(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn get_household(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<HouseholdResponse>, ApiError> {
    let command = GetHouseholdCommand {
        household_id: HouseholdId::from_uuid(household_id),
        requester_id: current_user.user_id(),
    };

    let household = state
        .households
        .get_household_for_user
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(Json(HouseholdResponse {
        id: household.id().to_string(),
        name: household.name().as_str().to_owned(),
        kind: household.kind().to_string(),
    }))
}

pub async fn add_household_member(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<AddHouseholdMemberRequest>,
) -> Result<StatusCode, ApiError> {
    let command = AddHouseholdMemberCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        member_email: request.email,
    };

    state
        .households
        .add_household_member
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn list_household_members(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<Json<Vec<HouseholdMemberResponse>>, ApiError> {
    let command = ListHouseholdMembersCommand {
        household_id: HouseholdId::from_uuid(household_id),
        requester_id: current_user.user_id(),
    };

    let members = state
        .households
        .list_household_members
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    let response = members
        .into_iter()
        .map(|member| HouseholdMemberResponse {
            user_id: member.user_id.to_string(),
            display_name: member.display_name.as_str().to_string(),
            role: member.role.to_string(),
        })
        .collect();

    Ok(Json(response))
}

pub async fn remove_household_member(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path((household_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = RemoveHouseholdMemberCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        member_id: UserId::from_uuid(member_id),
    };

    state
        .households
        .remove_household_member
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn rename_household(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
    Json(request): Json<RenameHouseholdRequest>,
) -> Result<StatusCode, ApiError> {
    let command = RenameHouseholdCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
        name: request.name,
    };
    state
        .households
        .rename_household
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn leave_household(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let command = LeaveHouseholdCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    state
        .households
        .leave_household
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}

pub async fn delete_household(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(household_id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    let command = DeleteHouseholdCommand {
        requester_id: current_user.user_id(),
        household_id: HouseholdId::from_uuid(household_id),
    };

    state
        .households
        .delete_household
        .execute(command)
        .await
        .map_err(ApiError::from)?;

    Ok(StatusCode::NO_CONTENT)
}
