export type Household = {
  id: string;
  name: string;
  kind: "personal" | "shared";
};

export type CreateHouseholdRequest = {
  name: string;
  kind: "personal" | "shared";
};

export type CreateHouseholdResponse = {
  id: string;
};

export type HouseholdMemberRole = "owner" | "member";
export type HouseholdMember = {
  user_id: string;
  display_name: string;
  role: HouseholdMemberRole;
};

export type RenameHouseholdRequest = {
  name: string;
};

export type AddHouseholdMemberRequest = {
  email: string;
};
