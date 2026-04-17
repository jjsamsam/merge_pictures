export type SupportedFileKind = "image" | "pdf" | "unknown";

export type QueueItem = {
  id: string;
  name: string;
  kind: Exclude<SupportedFileKind, "unknown">;
  path: string | null;
  size: number;
  pageCount: number | null;
  pixelWidth: number | null;
  pixelHeight: number | null;
  previewUrl: string | null;
  previewUrlKind: "blob" | "asset" | null;
};

const imageExtensions = [".jpg", ".jpeg", ".png", ".webp"];

export function detectSupportedFileKind(fileName: string): SupportedFileKind {
  const normalized = fileName.toLowerCase();

  if (normalized.endsWith(".pdf")) {
    return "pdf";
  }

  if (imageExtensions.some((extension) => normalized.endsWith(extension))) {
    return "image";
  }

  return "unknown";
}

export function getAcceptedInputTypes(): string {
  return [...imageExtensions, ".pdf"].join(",");
}

export function normalizeImportedFiles(files: FileList | File[]): QueueItem[] {
  const normalized: QueueItem[] = [];

  for (const file of Array.from(files)) {
    const kind = detectSupportedFileKind(file.name);
    if (kind === "unknown") {
      continue;
    }

    normalized.push({
      id: `${file.name}-${file.size}-${file.lastModified}`,
      name: file.name,
      kind,
      path: null,
      size: file.size,
      pageCount: null,
      pixelWidth: null,
      pixelHeight: null,
      previewUrl: kind === "image" ? URL.createObjectURL(file) : null,
      previewUrlKind: kind === "image" ? "blob" : null
    });
  }

  return normalized;
}

export function normalizeInspectedItems(
  items: Array<{
    path: string;
    name: string;
    kind: "image" | "pdf";
    size: number;
    pageCount?: number | null;
    pixelWidth?: number | null;
    pixelHeight?: number | null;
    previewUrl?: string | null;
  }>
): QueueItem[] {
  return items.map((item) => ({
    id: `${item.path}-${item.size}`,
    name: item.name,
    kind: item.kind,
    path: item.path,
    size: item.size,
    pageCount: item.pageCount ?? null,
    pixelWidth: item.pixelWidth ?? null,
    pixelHeight: item.pixelHeight ?? null,
    previewUrl: item.previewUrl ?? null,
    previewUrlKind: item.previewUrl ? "asset" : null
  }));
}

export function reorderItems<T>(items: T[], fromIndex: number, toIndex: number): T[] {
  if (fromIndex < 0 || fromIndex >= items.length) {
    return items;
  }

  if (toIndex < 0 || toIndex >= items.length) {
    return items;
  }

  const next = [...items];
  const [moved] = next.splice(fromIndex, 1);
  next.splice(toIndex, 0, moved);
  return next;
}

export function formatFileSize(size: number): string {
  if (size < 1024) {
    return `${size} B`;
  }

  if (size < 1024 * 1024) {
    return `${(size / 1024).toFixed(0)} KB`;
  }

  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

export function buildDefaultOutputName(date: Date): string {
  const year = date.getFullYear();
  const month = String(date.getMonth() + 1).padStart(2, "0");
  const day = String(date.getDate()).padStart(2, "0");
  const hour = String(date.getHours()).padStart(2, "0");
  const minute = String(date.getMinutes()).padStart(2, "0");
  const second = String(date.getSeconds()).padStart(2, "0");

  return `merged-${year}${month}${day}-${hour}${minute}${second}.pdf`;
}

export function ensurePdfFileName(value: string): string {
  const trimmed = value.trim();
  if (!trimmed) {
    return "merged.pdf";
  }

  return trimmed.toLowerCase().endsWith(".pdf") ? trimmed : `${trimmed}.pdf`;
}

export function hasReadyPaths(items: QueueItem[]): boolean {
  return items.length > 0 && items.every((item) => Boolean(item.path));
}

export function describeQueueItem(item: QueueItem): string {
  if (item.kind === "pdf") {
    return item.pageCount ? `${item.pageCount} page${item.pageCount === 1 ? "" : "s"}` : "PDF";
  }

  if (item.pixelWidth && item.pixelHeight) {
    return `${item.pixelWidth}×${item.pixelHeight}px image`;
  }

  return "Image";
}

export function revokePreviewUrls(items: QueueItem[]): void {
  for (const item of items) {
    if (item.previewUrl && item.previewUrlKind === "blob") {
      URL.revokeObjectURL(item.previewUrl);
    }
  }
}
