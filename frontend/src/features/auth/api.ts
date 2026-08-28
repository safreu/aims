import { apiJson, apiRequest } from "../../api/client";
import type {
  CurrentUser,
  LoginRequest,
  LoginResponse,
  RegisterRequest,
  RegisterResponse,
} from "./types";

export async function login(request: LoginRequest): Promise<LoginResponse> {
  return apiJson<LoginResponse>("/auth/login", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });
}

export async function register(
  request: RegisterRequest,
): Promise<RegisterResponse> {
  return apiJson<RegisterResponse>("/auth/register", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });
}

export async function getCurrentUser(): Promise<CurrentUser> {
  return apiJson<CurrentUser>("/auth/me");
}

export async function logout(): Promise<void> {
  await apiRequest("/auth/logout", { method: "POST" });
}
