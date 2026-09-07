import { ApiError } from "../../api/client";

export function isHouseholdAccessError(error: unknown): boolean {
  return (
    error instanceof ApiError && (error.status === 403 || error.status === 404)
  );
}
