import { useEffect, useId, useMemo, useRef, useState } from "react";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import {
  buildDefaultOutputName,
  describeQueueItem,
  ensurePdfFileName,
  formatFileSize,
  getAcceptedInputTypes,
  hasReadyPaths,
  normalizeImportedFiles,
  normalizeInspectedItems,
  revokePreviewUrls,
  reorderItems,
  type QueueItem
} from "./lib/files";

type NoticeTone = "neutral" | "success" | "error";
type PageSizePreset = "auto" | "a4" | "letter";
type ImageFitMode = "contain" | "cover" | "fill";

type InspectResponseItem = {
  path: string;
  name: string;
  kind: "image" | "pdf";
  size: number;
  pageCount?: number | null;
  pixelWidth?: number | null;
  pixelHeight?: number | null;
  previewUrl?: string | null;
};

const initialItems: QueueItem[] = [];

export default function App() {
  const inputId = useId();
  const [items, setItems] = useState<QueueItem[]>(initialItems);
  const [outputName, setOutputName] = useState(buildDefaultOutputName(new Date()));
  const [isPicking, setIsPicking] = useState(false);
  const [isMerging, setIsMerging] = useState(false);
  const [isDropActive, setIsDropActive] = useState(false);
  const [imagePageSize, setImagePageSize] = useState<PageSizePreset>("a4");
  const [imageMarginMm, setImageMarginMm] = useState(12);
  const [imageFitMode, setImageFitMode] = useState<ImageFitMode>("contain");
  const [notice, setNotice] = useState<string>("Choose files to build your merged PDF.");
  const [noticeTone, setNoticeTone] = useState<NoticeTone>("neutral");
  const itemsRef = useRef(items);

  const canMerge = useMemo(() => hasReadyPaths(items) && !isMerging, [items, isMerging]);

  const setStatus = (message: string, tone: NoticeTone = "neutral") => {
    setNotice(message);
    setNoticeTone(tone);
  };

  useEffect(() => {
    itemsRef.current = items;
  }, [items]);

  const onFilesSelected = (files: FileList | null) => {
    if (!files) {
      return;
    }

    const nextItems = normalizeImportedFiles(files);
    setItems((current) => [...current, ...nextItems]);
    setStatus(
      "Browser-added files can preview in the queue, but desktop merge requires the Tauri file picker.",
      "neutral"
    );
  };

  const appendInspectedPaths = async (paths: string[], source: "picker" | "drop") => {
    const inspected = await invoke<InspectResponseItem[]>("inspect_inputs", {
      request: { paths }
    });

    setItems((current) => [
      ...current,
      ...normalizeInspectedItems(
        inspected.map((item) => ({
          ...item,
          previewUrl:
            item.kind === "image" && item.path ? convertFileSrc(item.path) : null
        }))
      )
    ]);
    setStatus(
      `${inspected.length} files added to the merge queue from ${
        source === "drop" ? "drag and drop" : "the file picker"
      }.`,
      "success"
    );
  };

  useEffect(() => {
    let detach: (() => void) | undefined;
    let cancelled = false;

    async function attachDragDropListener() {
      try {
        const { getCurrentWindow } = await import("@tauri-apps/api/window");
        detach = await getCurrentWindow().onDragDropEvent(async (event) => {
          if (cancelled) {
            return;
          }

          if (event.payload.type === "enter" || event.payload.type === "over") {
            setIsDropActive(true);
            return;
          }

          if (event.payload.type === "leave") {
            setIsDropActive(false);
            return;
          }

          if (event.payload.type === "drop") {
            setIsDropActive(false);
            try {
              await appendInspectedPaths(event.payload.paths, "drop");
            } catch (error) {
              const message =
                error instanceof Error ? error.message : "Dropped files could not be imported.";
              setStatus(message, "error");
            }
          }
        });
      } catch {
        // Browser test/dev mode can run without a Tauri window bridge.
      }
    }

    void attachDragDropListener();

    return () => {
      cancelled = true;
      detach?.();
    };
  }, []);

  useEffect(() => {
    return () => {
      revokePreviewUrls(itemsRef.current);
    };
  }, []);

  const chooseFiles = async () => {
    setIsPicking(true);
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        directory: false,
        filters: [
          {
            name: "Supported files",
            extensions: ["jpg", "jpeg", "png", "webp", "pdf"]
          }
        ]
      });

      if (!selected) {
        setStatus("File selection was cancelled.", "neutral");
        return;
      }

      const paths = Array.isArray(selected) ? selected : [selected];
      await appendInspectedPaths(paths, "picker");
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Unable to open the desktop file picker.";
      setStatus(message, "error");
    } finally {
      setIsPicking(false);
    }
  };

  const mergeToPdf = async () => {
    if (!canMerge) {
      return;
    }

    setIsMerging(true);
    setStatus("Preparing output path...", "neutral");

    try {
      const { save } = await import("@tauri-apps/plugin-dialog");
      const finalName = ensurePdfFileName(outputName);
      const destination = await save({
        defaultPath: finalName,
        filters: [{ name: "PDF document", extensions: ["pdf"] }]
      });

      if (!destination) {
        setStatus("Export was cancelled before choosing a save path.", "neutral");
        return;
      }

      setOutputName(finalName);
      setStatus("Merging files into a single PDF...", "neutral");

      await invoke("merge_to_pdf", {
        request: {
          items: items
            .filter((item): item is QueueItem & { path: string } => Boolean(item.path))
            .map((item) => ({
              path: item.path,
              name: item.name,
              kind: item.kind,
              size: item.size
            })),
          outputPath: destination,
          imagePageSize,
          imageMarginMm,
          imageFitMode
        }
      });

      setStatus(`Merged PDF saved to ${destination}`, "success");
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "The PDF merge failed unexpectedly.";
      setStatus(message, "error");
    } finally {
      setIsMerging(false);
    }
  };

  const moveItem = (index: number, direction: -1 | 1) => {
    setItems((current) => reorderItems(current, index, index + direction));
  };

  return (
    <div className="shell">
      <div className="background-glow background-glow-left" />
      <div className="background-glow background-glow-right" />

      <header className="topbar">
        <div>
          <p className="eyebrow">Desktop PDF Merger</p>
          <h1>Merge images and PDFs into one clean document.</h1>
        </div>

        <div className="topbar-actions">
          <button
            className="button button-primary"
            onClick={chooseFiles}
            type="button"
          >
            {isPicking ? "Opening..." : "Add Files"}
          </button>
          <button
            className="button button-secondary"
            onClick={() => {
              revokePreviewUrls(items);
              setItems([]);
              setStatus("Queue cleared.", "neutral");
            }}
            type="button"
          >
            Clear All
          </button>
          <label className="button button-secondary" htmlFor={inputId}>
            Browser Demo Import
          </label>
          <input
            id={inputId}
            hidden
            multiple
            accept={getAcceptedInputTypes()}
            onChange={(event) => onFilesSelected(event.target.files)}
            type="file"
          />
        </div>
      </header>

      <main className="layout">
        <section className="panel hero-panel">
          <div className={`dropzone ${isDropActive ? "dropzone-active" : ""}`}>
            <p className="dropzone-title">Drop files here</p>
            <p className="dropzone-copy">
              Import JPG, PNG, WEBP, and PDF files. Rearrange the queue, then
              export a single merged PDF.
            </p>
            <p className="dropzone-hint">
              {isDropActive
                ? "Release to add files to the queue."
                : "Native desktop drag-and-drop is enabled."}
            </p>
            <div className="pill-row">
              <span className="pill">macOS</span>
              <span className="pill">Windows</span>
              <span className="pill">Semi-dark UI</span>
            </div>
          </div>
        </section>

        <section className="panel queue-panel">
          <div className="panel-heading">
            <div>
              <p className="panel-label">Merge Queue</p>
              <h2>{items.length} items ready</h2>
            </div>
          </div>

          <div className="queue-list">
            {items.length === 0 ? (
              <div className="empty-state">
                <p>No files loaded yet.</p>
                <span>Add images or PDFs to start building the merged document.</span>
              </div>
            ) : null}

            {items.map((item, index) => (
              <article className="queue-item" key={item.id}>
                <div className="queue-preview">
                  {item.kind === "image" && item.previewUrl ? (
                    <img
                      alt={`${item.name} preview`}
                      className="queue-preview-image"
                      src={item.previewUrl}
                    />
                  ) : (
                    <div className="queue-preview-pdf">
                      <div className="queue-preview-paper" />
                      <div className="queue-preview-label">
                        <span>PDF</span>
                        <strong>{item.pageCount ?? 1}</strong>
                      </div>
                    </div>
                  )}
                </div>
                <div className="queue-meta">
                  <strong>{item.name}</strong>
                  <div className="queue-tag-row">
                    <span className="queue-tag">{describeQueueItem(item)}</span>
                    <span className="queue-tag">{formatFileSize(item.size)}</span>
                  </div>
                  <span className="queue-path">
                    {item.path ? item.path : "Browser preview only"}
                  </span>
                </div>
                <div className="queue-actions">
                  <button
                    aria-label={`Move ${item.name} up`}
                    className="icon-button"
                    disabled={index === 0}
                    onClick={() => moveItem(index, -1)}
                    type="button"
                  >
                    ↑
                  </button>
                  <button
                    aria-label={`Move ${item.name} down`}
                    className="icon-button"
                    disabled={index === items.length - 1}
                    onClick={() => moveItem(index, 1)}
                    type="button"
                  >
                    ↓
                  </button>
                  <button
                    aria-label={`Remove ${item.name}`}
                    className="icon-button icon-button-danger"
                    onClick={() => {
                      setItems((current) => {
                        const next = current.filter((candidate) => candidate.id !== item.id);
                        revokePreviewUrls(current.filter((candidate) => candidate.id === item.id));
                        return next;
                      });
                      setStatus(`${item.name} removed from the queue.`, "neutral");
                    }}
                    type="button"
                  >
                    ×
                  </button>
                </div>
              </article>
            ))}
          </div>
        </section>

        <aside className="panel settings-panel">
          <div className="panel-heading">
            <div>
              <p className="panel-label">Output</p>
              <h2>Ready to export</h2>
            </div>
          </div>

          <label className="field">
            <span>Output file name</span>
            <input
              onChange={(event) => setOutputName(event.target.value)}
              type="text"
              value={outputName}
            />
          </label>

          <div className="summary-card">
            <div>
              <span className="summary-label">Supported inputs</span>
              <strong>JPG, PNG, WEBP, PDF</strong>
            </div>
            <div>
              <span className="summary-label">Merge order</span>
              <strong>Top to bottom queue</strong>
            </div>
            <div>
              <span className="summary-label">Output</span>
              <strong>Single PDF document</strong>
            </div>
          </div>

          <div className="summary-card">
            <div>
              <span className="summary-label">Image Page Layout</span>
              <strong>Applies to image inputs only</strong>
            </div>

            <label className="field field-compact">
              <span>Page size</span>
              <select
                className="select"
                onChange={(event) => setImagePageSize(event.target.value as PageSizePreset)}
                value={imagePageSize}
              >
                <option value="auto">Auto</option>
                <option value="a4">A4</option>
                <option value="letter">Letter</option>
              </select>
            </label>

            <label className="field field-compact">
              <span>Margins</span>
              <input
                max={40}
                min={0}
                onChange={(event) =>
                  setImageMarginMm(
                    Math.max(0, Math.min(40, Number.parseInt(event.target.value || "0", 10)))
                  )
                }
                type="range"
                value={imageMarginMm}
              />
              <strong>{imageMarginMm} mm</strong>
            </label>

            <label className="field field-compact">
              <span>Image fit</span>
              <select
                className="select"
                onChange={(event) => setImageFitMode(event.target.value as ImageFitMode)}
                value={imageFitMode}
              >
                <option value="contain">Contain</option>
                <option value="cover">Cover</option>
                <option value="fill">Fill</option>
              </select>
            </label>
          </div>

          <div className={`status-card status-card-${noticeTone}`} role="status">
            {notice}
          </div>

          <button
            className="button button-primary button-block"
            disabled={!canMerge}
            onClick={mergeToPdf}
            type="button"
          >
            {isMerging ? "Merging..." : "Merge To PDF"}
          </button>

          {!hasReadyPaths(items) && items.length > 0 ? (
            <p className="helper-copy">
              Desktop merge requires files chosen through the native Tauri file picker.
            </p>
          ) : null}
        </aside>
      </main>
    </div>
  );
}
