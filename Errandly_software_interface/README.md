# Errandly website

Next.js App Router + React + TypeScript. Static export, with no required backend.

## Develop

```sh
npm install
npm run dev
```

## Build

```sh
npm run build
```

Deploy the generated `out/` directory to any static hosting provider. Trailing-slash URLs are enabled.

## Product status

Errandly is in development. Download buttons intentionally lead to an honest coming-soon page, with no fake installer. Pricing, final requirements, support contacts, license and privacy terms must be finalized before the software launches. All demo data is fictional. The film and product interfaces are original concept previews.

## Files

- `app/page.tsx`: homepage
- `app/[...slug]/page.tsx`: supporting routes and editorial content
- `components/site.tsx`: navigation, footer, interactive product demo, video dialog, FAQ
- `app/globals.css`: responsive theme
- `public/`: original landscape, film, captions, poster, and SVG app icon
- `scripts/render-film.py`: reproducible concept film (requires Pillow and imageio-ffmpeg)

At launch, replace the disabled download control with a signed/notarized installer URL and publish verified system requirements. No email collection or purchase service is connected.

## Workspace interface

Open `/workspace/` for the alternate, full-screen chat interface. It includes searchable conversations, browser-local chat history and instructions, filename attachments, text export, and a stoppable three-stage response animation. Responses are simulated; there is no connected AI backend and attachment contents are not read. Small screens use collapsible navigation and context panels. Reduced-motion preferences disable animations.
