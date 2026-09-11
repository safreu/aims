import { getDiagnosticRecords, type DiagnosticRecord } from "./diagnosticsSink";

type ExportedLogRecord = {
  firstSeen: string;
  lastSeen: string;
  count: number;
  level: string;
  category: string;
  message: string;
  properties: Record<string, unknown>;
};

type DiagnosticExport = {
  exportedAt: string;
  app: {
    name: string;
  };
  environment: {
    userAgent: string;
    language: string;
    online: boolean;
  };
  logs: ExportedLogRecord[];
};

function serializeRecord(diagnostic: DiagnosticRecord): ExportedLogRecord {
  const { record } = diagnostic;
  return {
    firstSeen: new Date(diagnostic.firstSeen).toISOString(),
    lastSeen: new Date(diagnostic.lastSeen).toISOString(),
    count: diagnostic.count,
    level: record.level,
    category: record.category.join("."),
    message: record.message.map(String).join(""),
    properties: record.properties,
  };
}

export function exportDiagnostics(): void {
  const diagnostics: DiagnosticExport = {
    exportedAt: new Date().toISOString(),
    app: { name: "Aims" },
    environment: {
      userAgent: navigator.userAgent,
      language: navigator.language,
      online: navigator.onLine,
    },
    logs: getDiagnosticRecords().map(serializeRecord),
  };

  const blob = new Blob([JSON.stringify(diagnostics, null, 2)], {
    type: "application/json",
  });

  const url = URL.createObjectURL(blob);

  const timestamp = new Date().toISOString().replaceAll(":", "-");

  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = `aims-diagnostics-${timestamp}.json`;

  document.body.appendChild(anchor);
  anchor.click();
  anchor.remove();

  URL.revokeObjectURL(url);
}
