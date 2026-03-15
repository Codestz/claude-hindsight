/**
 * Image preview with expand-to-lightbox functionality.
 *
 * Renders base64 image data as an inline preview with:
 * - Size and dimension display
 * - EXPAND button to open fullscreen lightbox
 * - Size limit validation (5MB max)
 * - Base64 format validation
 * - Click-to-close lightbox overlay
 */

import { useState } from "react";

interface ImageData {
  mediaType: string;
  data: string;
}

interface ImagePreviewProps {
  img: ImageData;
  index: number;
}

const MAX_IMAGE_SIZE_KB = 5000; // 5MB limit
const BASE64_RE = /^[A-Za-z0-9+/\n\r]*={0,2}$/;

export function ImagePreview({ img, index }: ImagePreviewProps) {
  const [lightbox, setLightbox] = useState(false);
  const [dims, setDims] = useState<{ w: number; h: number } | null>(null);
  const sizeKb = Math.round(img.data.length * 0.75 / 1024);

  // Validate size
  if (sizeKb > MAX_IMAGE_SIZE_KB) {
    return (
      <div style={{ padding: "12px", background: "var(--bg-2)", border: "1px solid var(--border-1)", borderRadius: "var(--radius-md)", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
        Image too large ({sizeKb}KB, max {MAX_IMAGE_SIZE_KB}KB)
      </div>
    );
  }

  // Validate base64 format (check first 100 chars)
  if (!BASE64_RE.test(img.data.slice(0, 100))) {
    return (
      <div style={{ padding: "12px", background: "var(--bg-2)", border: "1px solid var(--border-1)", borderRadius: "var(--radius-md)", fontFamily: "var(--font-mono)", fontSize: "11px", color: "var(--text-3)" }}>
        Invalid image data
      </div>
    );
  }

  const src = `data:${img.mediaType};base64,${img.data}`;

  return (
    <>
      <div style={{
        borderRadius: "var(--radius-md)",
        overflow: "hidden",
        border: "1px solid var(--border-1)",
        background: "var(--bg-2)",
      }}>
        <img
          src={src}
          alt={`Image ${index + 1}`}
          onLoad={(e) => {
            const el = e.currentTarget;
            setDims({ w: el.naturalWidth, h: el.naturalHeight });
          }}
          style={{
            display: "block",
            maxWidth: "100%",
            maxHeight: "300px",
            objectFit: "contain",
            margin: "0 auto",
            background: "var(--bg-1)",
            imageRendering: "auto",
          }}
        />
        <div style={{
          padding: "4px 10px",
          fontFamily: "var(--font-mono)", fontSize: "9px",
          color: "var(--text-3)", borderTop: "1px solid var(--border-1)",
          display: "flex", justifyContent: "space-between", alignItems: "center",
        }}>
          <span>{img.mediaType} · {sizeKb}KB{dims ? ` · ${dims.w}×${dims.h}` : ""}</span>
          <button
            onClick={() => setLightbox(true)}
            style={{
              fontFamily: "var(--font-mono)", fontSize: "9px", fontWeight: 600,
              color: "var(--indigo)", background: "rgba(129,140,248,0.08)",
              border: "1px solid rgba(129,140,248,0.2)",
              borderRadius: "var(--radius-sm)",
              padding: "2px 8px", cursor: "pointer",
              transition: "all 0.1s",
            }}
          >
            EXPAND
          </button>
        </div>
      </div>

      {/* Lightbox overlay */}
      {lightbox && (
        <div
          onClick={() => setLightbox(false)}
          style={{
            position: "fixed", inset: 0,
            background: "rgba(0,0,0,0.85)",
            zIndex: 200,
            display: "flex", alignItems: "center", justifyContent: "center",
            cursor: "zoom-out",
            animation: "fadeIn 0.15s ease forwards",
          }}
        >
          <div
            onClick={(e) => e.stopPropagation()}
            style={{ position: "relative", maxWidth: "95vw", maxHeight: "95vh", cursor: "default" }}
          >
            <img
              src={src}
              alt={`Image ${index + 1} full`}
              style={{
                display: "block", maxWidth: "95vw", maxHeight: "90vh",
                objectFit: "contain", borderRadius: "var(--radius-md)",
                boxShadow: "var(--shadow-xl)",
              }}
            />
            <div style={{
              display: "flex", justifyContent: "space-between", alignItems: "center",
              marginTop: "8px", padding: "0 4px",
            }}>
              <span style={{ fontFamily: "var(--font-mono)", fontSize: "10px", color: "rgba(255,255,255,0.5)" }}>
                {img.mediaType} · {sizeKb}KB{dims ? ` · ${dims.w}×${dims.h} (transcript resolution)` : ""}
              </span>
              <button
                onClick={() => setLightbox(false)}
                style={{
                  fontFamily: "var(--font-mono)", fontSize: "10px", fontWeight: 600,
                  color: "var(--text-1)", background: "rgba(255,255,255,0.1)",
                  border: "1px solid rgba(255,255,255,0.15)",
                  borderRadius: "var(--radius-sm)",
                  padding: "4px 12px", cursor: "pointer",
                }}
              >
                CLOSE
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
