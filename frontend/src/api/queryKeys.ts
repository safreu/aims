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

    item: (householdId: string, itemId: string) =>
      ["households", householdId, "inventory", "items", itemId] as const,

    history: (householdId: string, itemId: string) =>
      [
        "households",
        householdId,
        "inventory",
        "items",
        itemId,
        "history",
      ] as const,

    qrActions: (householdId: string, itemId: string) =>
      [
        "households",
        householdId,
        "inventory",
        "items",
        itemId,
        "qr-actions",
      ] as const,
  },
  shopping: (householdId: string) =>
    ["households", householdId, "shopping"] as const,

  categories: (householdId: string) =>
    ["households", householdId, "categories"] as const,

  household: {
    detail: (householdId: string) => ["households", householdId] as const,

    members: (householdId: string) =>
      ["households", householdId, "members"] as const,
  },
};
