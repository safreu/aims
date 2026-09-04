export function subscribeToHouseholdEvents(
  householdId: string,
  onShoppingListChanged: () => void,
): EventSource {
  const eventSource = new EventSource(
    `/api/v1/households/${householdId}/events`,
  );
  eventSource.addEventListener("shopping_list_changed", () => {
    onShoppingListChanged();
  });

  return eventSource;
}
