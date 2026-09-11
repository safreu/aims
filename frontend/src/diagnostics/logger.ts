import { getLogger } from "@logtape/logtape";

export const apiLogger = getLogger(["aims", "api"]);
export const authLogger = getLogger(["aims", "auth"]);
export const scannerLogger = getLogger(["aims", "scanner"]);
export const sseLogger = getLogger(["aims", "sse"]);
export const appLogger = getLogger(["aims", "app"]);
