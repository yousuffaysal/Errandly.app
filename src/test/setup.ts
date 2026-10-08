import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";
import { cleanup } from "@testing-library/react";
import { clearMocks } from "@tauri-apps/api/mocks";

// jsdom lacks these browser APIs the app uses.
Element.prototype.scrollIntoView = () => {};
window.matchMedia ??= ((q: string) => ({ matches: false, media: q, addEventListener() {}, removeEventListener() {} })) as never;

afterEach(async () => {
  cleanup();
  // Let unmounting components unsubscribe from events before the mock goes away.
  await new Promise((r) => setTimeout(r, 0));
  clearMocks();
});
