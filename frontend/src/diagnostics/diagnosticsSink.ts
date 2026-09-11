import type { Sink, LogRecord } from "@logtape/logtape";

const MAX_DIAGNOSTIC_ENTRIES = 500;
const DEDUPLICTATION_WINDOW_MS = 30_000;

export type DiagnosticRecord = {
  record: LogRecord;
  count: number;
  firstSeen: number;
  lastSeen: number;
};

const diagnosticRecords: DiagnosticRecord[] = [];

export const diagnosticSink: Sink = (record) => {
  const now = record.timestamp;
  const lastRecord = diagnosticRecords[diagnosticRecords.length - 1];
  if (
    lastRecord !== undefined &&
    isSameRecord(lastRecord.record, record) &&
    now - lastRecord.lastSeen <= DEDUPLICTATION_WINDOW_MS
  ) {
    lastRecord.count += 1;
    lastRecord.lastSeen = now;
    return;
  }
  diagnosticRecords.push({
    record,
    count: 1,
    firstSeen: now,
    lastSeen: now,
  });

  if (diagnosticRecords.length > MAX_DIAGNOSTIC_ENTRIES)
    diagnosticRecords.shift();
};

function isSameRecord(first: LogRecord, second: LogRecord): boolean {
  return (
    first.level === second.level &&
    first.category.join(".") === second.category.join(".") &&
    first.rawMessage === second.rawMessage &&
    JSON.stringify(first.properties) === JSON.stringify(second.properties)
  );
}

export function getDiagnosticRecords(): readonly DiagnosticRecord[] {
  return [...diagnosticRecords];
}

export function clearDiagnosticRecords(): void {
  diagnosticRecords.length = 0;
}
