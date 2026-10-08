export const basename = (p: string) => p.split("/").filter(Boolean).pop() ?? p;

/** `path` relative to `root`, for display. */
export const relative = (root: string, path: string) =>
  path.startsWith(root + "/") ? path.slice(root.length + 1) : path;
