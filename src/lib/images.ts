// Resolve an image reference from a Markdown document to an absolute file path.
//
// Only plain relative or absolute file paths are handled here. Anything with a scheme
// (https:, data:, asset:, file:...) is left alone, and only real image extensions are
// accepted so a document can't point an <img> at an arbitrary file. Whether the resulting
// path may actually be shown is decided by Rust: the asset protocol is scoped to the
// document's folder, so `../` tricks simply fail to load.

const IMAGE_EXTENSIONS = new Set(["png", "jpg", "jpeg", "gif", "webp", "avif", "bmp"]);

export function resolveLocalImage(docPath: string | null, src: string): string | null {
  if (!docPath) return null; // an unsaved document has no folder to look in
  const s = src.trim();
  if (!s || s.startsWith("#") || s.startsWith("//") || /^[a-z][a-z0-9+.-]*:/i.test(s)) {
    return null;
  }

  let rel = s.split(/[?#]/)[0];
  try {
    rel = decodeURIComponent(rel);
  } catch {
    /* keep the raw text */
  }

  const ext = rel.includes(".") ? rel.slice(rel.lastIndexOf(".") + 1).toLowerCase() : "";
  if (!IMAGE_EXTENSIONS.has(ext)) return null;

  const dir = docPath.replace(/[/\\][^/\\]*$/, "");
  const joined = rel.startsWith("/") ? rel : `${dir}/${rel}`;

  // Collapse "." and ".." so the path is canonical before it is handed to the scope check.
  const parts: string[] = [];
  for (const seg of joined.split("/")) {
    if (seg === "" || seg === ".") continue;
    if (seg === "..") parts.pop();
    else parts.push(seg);
  }
  return "/" + parts.join("/");
}
