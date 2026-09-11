import type { QrAction } from "../types";

export async function createQrCardBlob(
  qrCanvas: HTMLCanvasElement,
  itemName: string,
  action: QrAction,
): Promise<Blob> {
  const width = 800;
  const height = 1000;

  const canvas = document.createElement("canvas");
  canvas.width = width;
  canvas.height = height;

  const context = canvas.getContext("2d");

  if (context === null) {
    throw new Error("Could not create canvas context");
  }

  context.fillStyle = "#ffffff";
  context.fillRect(0, 0, width, height);

  context.fillStyle = "#111111";
  context.textAlign = "center";

  context.font = "600 42px sans-serif";
  context.fillText(itemName, width / 2, 90);

  const qrSize = 560;
  const qrX = (width - qrSize) / 2;
  const qrY = 170;

  context.drawImage(qrCanvas, qrX, qrY, qrSize, qrSize);

  context.font = "600 36px sans-serif";

  const actionText =
    action.kind === "increase"
      ? `Increase Stock +${action.amount}`
      : `Decrease Stock -${action.amount}`;

  context.fillText(actionText, width / 2, 810);

  return new Promise((resolve, reject) => {
    canvas.toBlob((blob) => {
      if (blob === null) {
        reject(new Error("Failed to create QR image"));
        return;
      }
      resolve(blob);
    }, "image/png");
  });
}

function createFileName(itemName: string, action: QrAction): string {
  const safeName = itemName
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, "-")
    .replace(/^-+|-+$/g, "");

  return `${safeName || "inventory-item"}-${action.kind}-qr.png`;
}

function downloadBlob(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob);

  const link = document.createElement("a");
  link.href = url;
  link.download = fileName;

  document.body.appendChild(link);
  link.click();
  link.remove();

  URL.revokeObjectURL(url);
}

export async function shareQrCard(
  qrCanvas: HTMLCanvasElement,
  itemName: string,
  action: QrAction,
): Promise<"shared" | "downloaded" | "cancelled"> {
  const blob = await createQrCardBlob(qrCanvas, itemName, action);

  const fileName = createFileName(itemName, action);

  const file = new File([blob], fileName, { type: "image/png" });

  if (
    typeof navigator.share === "function" &&
    navigator.canShare?.({ files: [file] })
  ) {
    try {
      await navigator.share({
        title: itemName,
        files: [file],
      });

      return "shared";
    } catch (error) {
      if (error instanceof DOMException && error.name === "AbortError") {
        return "cancelled";
      }

      throw error;
    }
  }
  downloadBlob(blob, fileName);

  return "downloaded";
}
