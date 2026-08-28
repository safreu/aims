import { apiJson } from "../../api/client";
import type {
  CreateHouseholdRequest,
  CreateHouseholdResponse,
  Household,
} from "./types";

export async function getHouseholds(): Promise<Household[]> {
  return apiJson<Household[]>("/households");
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
