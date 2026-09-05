import { apiJson, apiRequest } from "../../api/client";
import type {
  AddHouseholdMemberRequest,
  CreateHouseholdRequest,
  CreateHouseholdResponse,
  Household,
  HouseholdMember,
  RenameHouseholdRequest,
} from "./types";

export async function getHouseholds(): Promise<Household[]> {
  return apiJson<Household[]>("/households");
}

export async function getHousehold(householdId: string): Promise<Household> {
  return apiJson<Household>(`/households/${householdId}`);
}

export async function createHousehold(
  request: CreateHouseholdRequest,
): Promise<CreateHouseholdResponse> {
  return apiJson<CreateHouseholdResponse>("/households", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function renameHousehold(
  householdId: string,
  request: RenameHouseholdRequest,
): Promise<void> {
  await apiRequest(`/households/${householdId}`, {
    method: "PATCH",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function getHouseholdMembers(
  householdId: string,
): Promise<HouseholdMember[]> {
  return apiJson<HouseholdMember[]>(`/households/${householdId}/members`);
}

export async function addHouseholdMembers(
  householdId: string,
  request: AddHouseholdMemberRequest,
): Promise<void> {
  await apiRequest(`/households/${householdId}/members`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
}

export async function removeHouseholdMember(
  householdId: string,
  memberId: string,
): Promise<void> {
  await apiRequest(`/households/${householdId}/members/${memberId}`, {
    method: "DELETE",
  });
}
