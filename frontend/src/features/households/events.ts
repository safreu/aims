export function createHouseholdEventSource(householdId: string): EventSource {
  return new EventSource(`/api/v1/households/${householdId}/events`);
}
