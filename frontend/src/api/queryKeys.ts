export const queryKeys = {
  devices: (householdId: string) =>
    ["households", householdId, "devices"] as const,

  inventory: {
    all: (householdId: string) =>
      ["households", householdId, "inventory"] as const,

    active: (householdId: string) =>
      ["households", householdId, "inventory", "active"] as const,

    archived: (householdId: string) =>
      ["households", householdId, "inventory", "archived"] as const,
  },
  shopping: (householdId: string) =>
    ["households", householdId, "shopping"] as const,

  categories: (householdId: string) =>
    ["households", householdId, "categories"] as const,
};
