use crate::{
    modules::households::application::{
        AddHouseholdMemberError, CreateHouseholdError, DeleteHouseholdError, GetHouseholdError,
        LeaveHouseholdError, ListHouseholdMembersError, RemoveHouseholdMemberError,
        RenameHouseholdError, SubscribeHouseholdEventsError,
    },
    shared::api::error::ApiError,
};

impl From<CreateHouseholdError> for ApiError {
    fn from(error: CreateHouseholdError) -> Self {
        match error {
            CreateHouseholdError::Internal(_) => Self::internal_error(),
            CreateHouseholdError::InvalidName => {
                ApiError::bad_request("invalid_household_name", "The household name is invalid")
            }
            CreateHouseholdError::PersonalHouseholdAlreadyExists => ApiError::conflict(
                "personal_household_already_exists",
                "A personal household already exists",
            ),
        }
    }
}

impl From<GetHouseholdError> for ApiError {
    fn from(error: GetHouseholdError) -> Self {
        match error {
            GetHouseholdError::HouseholdAccess(error) => error.into(),
            GetHouseholdError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<AddHouseholdMemberError> for ApiError {
    fn from(error: AddHouseholdMemberError) -> Self {
        match error {
            AddHouseholdMemberError::InvalidEmail => {
                ApiError::bad_request("invalid_email", "The email address is invalid")
            }

            AddHouseholdMemberError::HouseholdNotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }

            AddHouseholdMemberError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permission to modify this household",
            ),

            AddHouseholdMemberError::PersonalHousehold => ApiError::conflict(
                "personal_household_does_not_support_members",
                "Members cannot be added to a personal household",
            ),

            AddHouseholdMemberError::UserNotFound => {
                ApiError::not_found("user_not_found", "The user was not found")
            }

            AddHouseholdMemberError::MemberAlreadyExists => ApiError::conflict(
                "household_member_already_exists",
                "The user is already a member of this household",
            ),

            AddHouseholdMemberError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<ListHouseholdMembersError> for ApiError {
    fn from(error: ListHouseholdMembersError) -> Self {
        match error {
            ListHouseholdMembersError::HouseholdAccess(error) => error.into(),
            ListHouseholdMembersError::NotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            ListHouseholdMembersError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<RemoveHouseholdMemberError> for ApiError {
    fn from(error: RemoveHouseholdMemberError) -> Self {
        match error {
            RemoveHouseholdMemberError::HouseholdAccess(error) => error.into(),
            RemoveHouseholdMemberError::MemberNotFound => ApiError::not_found(
                "household_member_not_found",
                "The household member was not found",
            ),
            RemoveHouseholdMemberError::OwnerCannotBeRemoved => ApiError::conflict(
                "household_owner_cannot_be_removed",
                "The household owner cannot be removed",
            ),
            RemoveHouseholdMemberError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<RenameHouseholdError> for ApiError {
    fn from(error: RenameHouseholdError) -> Self {
        match error {
            RenameHouseholdError::Forbidden => ApiError::forbidden(
                "household_access_forbidden",
                "You do not have permission to modify this household",
            ),
            RenameHouseholdError::InvalidName => {
                ApiError::bad_request("invalid_household_name", "The household name is invalid")
            }
            RenameHouseholdError::NotFound => {
                ApiError::not_found("household_not_found", "The household was not found")
            }
            RenameHouseholdError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<LeaveHouseholdError> for ApiError {
    fn from(error: LeaveHouseholdError) -> Self {
        match error {
            LeaveHouseholdError::HouseholdAccess(error) => error.into(),
            LeaveHouseholdError::OwnerCannotLeave => ApiError::conflict(
                "household_owner_cannot_leave",
                "The household owner cannot leave",
            ),
            LeaveHouseholdError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<DeleteHouseholdError> for ApiError {
    fn from(error: DeleteHouseholdError) -> Self {
        match error {
            DeleteHouseholdError::HouseholdAccess(error) => error.into(),
            DeleteHouseholdError::HouseholdHasOtherMembers => ApiError::conflict(
                "household_owner_cannot_delete_when_not_alone",
                "The household owner cannot leave as long as other members are part of the household",
            ),
            DeleteHouseholdError::Internal(_) => ApiError::internal_error(),
        }
    }
}

impl From<SubscribeHouseholdEventsError> for ApiError {
    fn from(value: SubscribeHouseholdEventsError) -> Self {
        match value {
            SubscribeHouseholdEventsError::HouseholdAccess(error) => error.into(),
            SubscribeHouseholdEventsError::Internal(_) => ApiError::internal_error(),
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::StatusCode;

    use crate::shared::application::InternalError;

    use super::*;

    #[test]
    fn invalid_household_name_maps_to_bad_request() {
        let error = ApiError::from(CreateHouseholdError::InvalidName);

        assert_eq!(error.status(), StatusCode::BAD_REQUEST);
        assert_eq!(error.code(), "invalid_household_name")
    }

    #[test]
    fn existing_personal_household_maps_to_conflict() {
        let error = ApiError::from(CreateHouseholdError::PersonalHouseholdAlreadyExists);

        assert_eq!(error.status(), StatusCode::CONFLICT);
        assert_eq!(error.code(), "personal_household_already_exists")
    }

    #[test]
    fn internal_household_error_maps_to_internal_server_error() {
        let error = ApiError::from(CreateHouseholdError::Internal(InternalError::Failed));

        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.code(), "internal_error")
    }
}
