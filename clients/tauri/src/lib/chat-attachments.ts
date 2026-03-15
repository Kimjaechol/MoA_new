/**
 * Chat attachment pre-processing module.
 *
 * Routes uploaded files through the correct processing pipeline
 * before sending content to the LLM via chat API.
 *
 * Routing logic:
 * - Image (png/jpg/gif/webp)       → multimodal (base64 image)
 * - Text PDF                       → pymupdf text extraction → text
 * - Image PDF (small)              → pymupdf page→image → multimodal
 * - Image PDF (large)              → R2 → Upstage Document Parser → markdown
 * - HWP/HWPX                       → Hancom Document Viewer API → markdown
 * - DOCX/XLSX/PPTX                 → local text extraction → text
 *
 * "Large image PDF" threshold:
 *   ≥7 files OR single file ≥10 pages OR (≥3 files AND each ≥3 pages)
 */

import type { Locale } from "./i18n";
import { apiClient } from "./api";

// ── Types ────────────────────────────────────────────────────────

export interface AttachmentFile {
  file: File;
  id: string;
  name: string;
  type: FileCategory;
  /** Total pages (for PDFs, filled after classification) */
  pageCount?: number;
  /** Whether a PDF is image-based (filled after classification) */
  isImagePdf?: boolean;
}

export type FileCategory =
  | "image"
  | "text_pdf"
  | "image_pdf_small"
  | "image_pdf_large"
  | "hwp"
  | "office"
  | "unsupported";

export interface ProcessedAttachment {
  /** Content type sent to LLM */
  mode: "text" | "multimodal_image" | "markdown";
  /** Extracted text/markdown content */
  content: string;
  /** Base64 image data (for multimodal) */
  images?: string[];
  /** Original filename */
  filename: string;
}

export interface AttachmentProcessProgress {
  filename: string;
  step: string;
  percent: number;
}

// ── Constants ────────────────────────────────────────────────────

const IMAGE_EXTENSIONS = [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp"];
const PDF_EXTENSION = ".pdf";
const HWP_EXTENSIONS = [".hwp", ".hwpx"];
const OFFICE_EXTENSIONS = [".doc", ".docx", ".xls", ".xlsx", ".ppt", ".pptx"];
const DOCUMENT_EXTENSIONS = [PDF_EXTENSION, ...HWP_EXTENSIONS, ...OFFICE_EXTENSIONS];

export const SUPPORTED_CHAT_EXTENSIONS = [
  ...IMAGE_EXTENSIONS,
  ...DOCUMENT_EXTENSIONS,
];

// Large image PDF thresholds
const LARGE_FILE_COUNT_THRESHOLD = 7;
const LARGE_PAGE_THRESHOLD = 10;
const MULTI_FILE_COUNT_THRESHOLD = 3;
const MULTI_FILE_PAGE_THRESHOLD = 3;

// ── Helpers ──────────────────────────────────────────────────────

function getExtension(filename: string): string {
  const dot = filename.lastIndexOf(".");
  return dot >= 0 ? filename.substring(dot).toLowerCase() : "";
}

function generateId(): string {
  return `attach_${Date.now()}_${Math.random().toString(36).substring(2, 8)}`;
}

// ── Classification ───────────────────────────────────────────────

export function classifyFile(file: File): FileCategory {
  const ext = getExtension(file.name);

  if (IMAGE_EXTENSIONS.includes(ext)) return "image";
  if (HWP_EXTENSIONS.includes(ext)) return "hwp";
  if (OFFICE_EXTENSIONS.includes(ext)) return "office";
  if (ext === PDF_EXTENSION) return "text_pdf"; // will be reclassified after PDF analysis
  return "unsupported";
}

export function isDocumentFile(file: File): boolean {
  const ext = getExtension(file.name);
  return DOCUMENT_EXTENSIONS.includes(ext);
}

/**
 * Check if a set of image PDF attachments qualifies as "large"
 * and should be routed through Upstage Document Parser.
 *
 * Criteria:
 * - ≥7 image PDF files, OR
 * - any single image PDF ≥10 pages, OR
 * - ≥3 image PDF files AND each ≥3 pages
 */
export function isLargeImagePdfBatch(imagePdfs: AttachmentFile[]): boolean {
  if (imagePdfs.length >= LARGE_FILE_COUNT_THRESHOLD) return true;
  if (imagePdfs.some((f) => (f.pageCount ?? 0) >= LARGE_PAGE_THRESHOLD)) return true;
  if (
    imagePdfs.length >= MULTI_FILE_COUNT_THRESHOLD &&
    imagePdfs.every((f) => (f.pageCount ?? 0) >= MULTI_FILE_PAGE_THRESHOLD)
  ) {
    return true;
  }
  return false;
}

// ── Create attachment from file ──────────────────────────────────

export function createAttachment(file: File): AttachmentFile {
  return {
    file,
    id: generateId(),
    name: file.name,
    type: classifyFile(file),
  };
}

// ── Processing functions ─────────────────────────────────────────

/** Convert image file to base64 for multimodal LLM */
async function processImage(file: File): Promise<ProcessedAttachment> {
  return new Promise((resolve, reject) => {
    const reader = new FileReader();
    reader.onload = () => {
      const base64 = (reader.result as string).split(",")[1]; // strip data:...;base64,
      resolve({
        mode: "multimodal_image",
        content: "",
        images: [base64],
        filename: file.name,
      });
    };
    reader.onerror = () => reject(new Error(`Failed to read image: ${file.name}`));
    reader.readAsDataURL(file);
  });
}

/** Extract text from digital PDF via PyMuPDF (Tauri command) */
async function processTextPdf(
  file: File,
  tauriInvoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null,
): Promise<ProcessedAttachment> {
  if (!tauriInvoke) {
    return { mode: "text", content: `[PDF file: ${file.name}]`, filename: file.name };
  }

  const base64 = await fileToBase64(file);
  const tempPath = (await tauriInvoke("write_temp_file", {
    base64Data: base64,
    extension: "pdf",
  })) as string;

  try {
    const result = (await tauriInvoke("convert_pdf_local", {
      filePath: tempPath,
    })) as { success: boolean; html: string; markdown: string; page_count: number };

    if (result.success && result.markdown) {
      return { mode: "markdown", content: result.markdown, filename: file.name };
    }
    return { mode: "text", content: `[PDF: ${file.name} - extraction failed]`, filename: file.name };
  } finally {
    tauriInvoke("cleanup_temp_file", { filePath: tempPath }).catch(() => {});
  }
}

/** Convert image PDF pages to images for multimodal (small batch) */
async function processImagePdfSmall(
  file: File,
  tauriInvoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null,
): Promise<ProcessedAttachment> {
  if (!tauriInvoke) {
    return { mode: "text", content: `[Image PDF: ${file.name}]`, filename: file.name };
  }

  const base64 = await fileToBase64(file);
  const tempPath = (await tauriInvoke("write_temp_file", {
    base64Data: base64,
    extension: "pdf",
  })) as string;

  try {
    // Use pymupdf to render pages as images
    const result = (await tauriInvoke("convert_pdf_to_images", {
      filePath: tempPath,
    })) as { success: boolean; images: string[]; page_count: number };

    if (result.success && result.images?.length > 0) {
      return {
        mode: "multimodal_image",
        content: "",
        images: result.images,
        filename: file.name,
      };
    }

    // Fallback: try text extraction
    return processTextPdf(file, tauriInvoke);
  } finally {
    tauriInvoke("cleanup_temp_file", { filePath: tempPath }).catch(() => {});
  }
}

/** Process large image PDF via R2 → Upstage Document Parser → markdown */
async function processImagePdfLarge(
  file: File,
  _locale: Locale,
  onProgress?: (step: string) => void,
): Promise<ProcessedAttachment> {
  const serverUrl = apiClient.getRelayUrl();
  const token = apiClient.getToken();

  // Step 1: Get pre-signed R2 upload URL
  onProgress?.("upload_url");
  const urlResp = await fetch(`${serverUrl}/api/document/upload-url`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({
      filename: file.name,
      content_type: file.type || "application/pdf",
      estimated_pages: 1,
    }),
  });

  if (!urlResp.ok) {
    const data = await urlResp.json().catch(() => ({ error: "Failed to get upload URL" }));
    throw new Error(data.error || `Upload URL failed (${urlResp.status})`);
  }

  const { upload_url, object_key } = await urlResp.json();

  // Step 2: Upload to R2
  onProgress?.("uploading");
  const uploadResp = await fetch(upload_url, {
    method: "PUT",
    headers: { "Content-Type": file.type || "application/pdf" },
    body: file,
  });

  if (!uploadResp.ok) {
    throw new Error(`R2 upload failed (HTTP ${uploadResp.status})`);
  }

  // Step 3: Process via Upstage OCR
  onProgress?.("ocr");
  const processResp = await fetch(`${serverUrl}/api/document/process-r2`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
      ...(token ? { Authorization: `Bearer ${token}` } : {}),
    },
    body: JSON.stringify({ object_key, filename: file.name, estimated_pages: 1 }),
  });

  if (!processResp.ok) {
    const data = await processResp.json().catch(() => ({ error: "OCR failed" }));
    throw new Error(data.error || `Upstage OCR failed (${processResp.status})`);
  }

  const result = await processResp.json();
  return {
    mode: "markdown",
    content: result.markdown || result.html || "",
    filename: file.name,
  };
}

/** Process HWP/HWPX via Hancom Document Viewer API */
async function processHwp(file: File): Promise<ProcessedAttachment> {
  const formData = new FormData();
  formData.append("file", file);

  const serverUrl = apiClient.getServerUrl();
  const token = apiClient.getToken();

  const response = await fetch(`${serverUrl}/api/document/process`, {
    method: "POST",
    headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    body: formData,
  });

  if (!response.ok) {
    const data = await response.json().catch(() => ({ error: "HWP processing failed" }));
    throw new Error(data.error || `HWP processing failed (${response.status})`);
  }

  const result = await response.json();
  return {
    mode: "markdown",
    content: result.markdown || result.html || "",
    filename: file.name,
  };
}

/** Process Office documents (DOCX/XLSX/PPTX) via local extraction */
async function processOffice(file: File): Promise<ProcessedAttachment> {
  const formData = new FormData();
  formData.append("file", file);

  const serverUrl = apiClient.getServerUrl();
  const token = apiClient.getToken();

  const response = await fetch(`${serverUrl}/api/document/process`, {
    method: "POST",
    headers: { ...(token ? { Authorization: `Bearer ${token}` } : {}) },
    body: formData,
  });

  if (!response.ok) {
    const data = await response.json().catch(() => ({ error: "Office processing failed" }));
    throw new Error(data.error || `Office processing failed (${response.status})`);
  }

  const result = await response.json();
  return {
    mode: "markdown",
    content: result.markdown || result.html || "",
    filename: file.name,
  };
}

// ── Main processing entry point ──────────────────────────────────

export interface ProcessOptions {
  locale: Locale;
  tauriInvoke: ((cmd: string, args?: Record<string, unknown>) => Promise<unknown>) | null;
  onProgress?: (info: AttachmentProcessProgress) => void;
}

/**
 * Process a single attachment and return content ready for LLM.
 */
export async function processAttachment(
  attachment: AttachmentFile,
  options: ProcessOptions,
): Promise<ProcessedAttachment> {
  const { locale, tauriInvoke, onProgress } = options;
  const progress = (step: string) =>
    onProgress?.({ filename: attachment.name, step, percent: 50 });

  switch (attachment.type) {
    case "image":
      progress("reading");
      return processImage(attachment.file);

    case "text_pdf":
      progress("extracting");
      return processTextPdf(attachment.file, tauriInvoke);

    case "image_pdf_small":
      progress("converting_pages");
      return processImagePdfSmall(attachment.file, tauriInvoke);

    case "image_pdf_large":
      return processImagePdfLarge(attachment.file, locale, (step) => progress(step));

    case "hwp":
      progress("hancom_converting");
      return processHwp(attachment.file);

    case "office":
      progress("office_converting");
      return processOffice(attachment.file);

    default:
      return { mode: "text", content: `[Unsupported file: ${attachment.name}]`, filename: attachment.name };
  }
}

/**
 * Build a chat message string from processed attachments + user text.
 */
export function buildChatMessage(
  userText: string,
  processed: ProcessedAttachment[],
): { message: string; images: string[] } {
  const textParts: string[] = [];
  const allImages: string[] = [];

  for (const p of processed) {
    if (p.mode === "multimodal_image" && p.images?.length) {
      allImages.push(...p.images);
      if (p.filename) {
        textParts.push(`[Attached image: ${p.filename}]`);
      }
    } else if (p.content) {
      textParts.push(`--- ${p.filename} ---\n${p.content}\n--- end ---`);
    }
  }

  if (userText.trim()) {
    textParts.push(userText);
  }

  return {
    message: textParts.join("\n\n"),
    images: allImages,
  };
}

// ── Utility ──────────────────────────────────────────────────────

async function fileToBase64(file: File): Promise<string> {
  const arrayBuf = await file.arrayBuffer();
  const bytes = new Uint8Array(arrayBuf);
  const chunkSize = 8192;
  let binaryStr = "";
  for (let i = 0; i < bytes.length; i += chunkSize) {
    const chunk = bytes.subarray(i, i + chunkSize);
    binaryStr += String.fromCharCode.apply(null, Array.from(chunk));
  }
  return btoa(binaryStr);
}

/** Estimate page count for a PDF without full parsing (file size heuristic) */
export function estimatePageCount(file: File): number {
  // Rough heuristic: ~100KB per page for image PDFs, ~20KB for text PDFs
  return Math.max(1, Math.ceil(file.size / 100_000));
}

/** Get human-readable file size */
export function formatFileSize(bytes: number): string {
  if (bytes < 1024) return `${bytes}B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)}KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)}MB`;
}
