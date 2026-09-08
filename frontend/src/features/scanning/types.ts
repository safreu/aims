export type QrActionKind = "increase" | "decrease";

export type QrAction = {
  id: string;
  item_id: string;
  kind: QrActionKind;
  amount: number;
};
