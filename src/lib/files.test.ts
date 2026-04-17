import {
  buildDefaultOutputName,
  describeQueueItem,
  detectSupportedFileKind,
  ensurePdfFileName,
  hasReadyPaths,
  reorderItems
} from "./files";

describe("detectSupportedFileKind", () => {
  it("detects PDFs and images", () => {
    expect(detectSupportedFileKind("deck.PDF")).toBe("pdf");
    expect(detectSupportedFileKind("photo.webp")).toBe("image");
    expect(detectSupportedFileKind("notes.txt")).toBe("unknown");
  });
});

describe("reorderItems", () => {
  it("moves an item inside the array", () => {
    expect(reorderItems(["a", "b", "c"], 0, 2)).toEqual(["b", "c", "a"]);
  });

  it("keeps the array unchanged for invalid moves", () => {
    expect(reorderItems(["a", "b"], 0, 5)).toEqual(["a", "b"]);
  });
});

describe("buildDefaultOutputName", () => {
  it("creates a deterministic file name", () => {
    const value = buildDefaultOutputName(new Date(2026, 3, 17, 9, 8, 7));
    expect(value).toBe("merged-20260417-090807.pdf");
  });
});

describe("ensurePdfFileName", () => {
  it("adds the pdf extension when needed", () => {
    expect(ensurePdfFileName("result")).toBe("result.pdf");
    expect(ensurePdfFileName("final.PDF")).toBe("final.PDF");
  });
});

describe("hasReadyPaths", () => {
  it("requires every queue item to have a path", () => {
    expect(
      hasReadyPaths([
        {
          id: "1",
          name: "a.pdf",
          kind: "pdf",
          path: "/tmp/a.pdf",
          size: 1,
          pageCount: null,
          pixelWidth: null,
          pixelHeight: null,
          previewUrl: null,
          previewUrlKind: null
        },
        {
          id: "2",
          name: "b.jpg",
          kind: "image",
          path: null,
          size: 1,
          pageCount: null,
          pixelWidth: null,
          pixelHeight: null,
          previewUrl: null,
          previewUrlKind: null
        }
      ])
    ).toBe(false);
  });
});

describe("describeQueueItem", () => {
  it("formats PDF and image metadata", () => {
    expect(
      describeQueueItem({
        id: "1",
        name: "a.pdf",
        kind: "pdf",
        path: "/tmp/a.pdf",
        size: 1,
        pageCount: 3,
        pixelWidth: null,
        pixelHeight: null,
        previewUrl: null,
        previewUrlKind: null
      })
    ).toBe("3 pages");

    expect(
      describeQueueItem({
        id: "2",
        name: "b.png",
        kind: "image",
        path: "/tmp/b.png",
        size: 1,
        pageCount: null,
        pixelWidth: 1600,
        pixelHeight: 900,
        previewUrl: null,
        previewUrlKind: null
      })
    ).toBe("1600×900px image");
  });
});
