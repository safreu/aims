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
