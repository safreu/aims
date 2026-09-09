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

  const response = await fetch(`${API_BASE_URL}${path}`, {
    ...requestOptions,
    credentials: "include",
  });

  if (response.status === 401 && handleUnauthorized) {
    unauthorizedHandler?.();
  }

  if (!response.ok) {
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
