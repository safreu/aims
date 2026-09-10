import { apiLogger } from "../diagnostics/logger";

const API_BASE_URL = "/api/v1";

let unauthorizedHandler: (() => void) | null = null;

export class ApiError extends Error {
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

export function setUnauthorizedHandler(handler: (() => void) | null): void {
  unauthorizedHandler = handler;
}

type ApiRequestOptions = RequestInit & {
  handleUnauthorized?: boolean;
};

export async function apiRequest(
  path: string,
  options?: ApiRequestOptions,
): Promise<Response> {
  const { handleUnauthorized = true, ...requestOptions } = options ?? {};
  const method = requestOptions.method ?? "GET";

  let response: Response;
  try {
    response = await fetch(`${API_BASE_URL}${path}`, {
      ...requestOptions,
      credentials: "include",
    });
  } catch (error) {
    apiLogger.error("API request failed before receiving a response", {
      method,
      path,
      error: getErrorMessage(error),
    });

    throw error;
  }

  if (response.status === 401 && handleUnauthorized) {
    unauthorizedHandler?.();
  }

  if (!response.ok) {
    const details = {
      method,
      path,
      status: response.status,
      statusText: response.statusText,
    };

    if (response.status >= 500) {
      apiLogger.error("API request returned {status}", details);
    } else {
      apiLogger.warn("API request returned {status}", details);
    }

    throw new ApiError(
      response.status,
      `Request failed with status ${response.status}`,
    );
  }

  return response;
}

export async function apiJson<T>(
  path: string,
  options?: RequestInit,
): Promise<T> {
  const response = await apiRequest(path, options);

  return response.json() as Promise<T>;
}

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;

  return String(error);
}
