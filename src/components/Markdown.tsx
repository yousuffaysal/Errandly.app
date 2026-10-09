import { Fragment, type ReactNode } from "react";

/**
 * Shows a reply's light Markdown (headings, lists, **bold**, *italic*,
 * `code`, code blocks) as formatted text. Built from React elements, never
 * raw HTML, so nothing in a reply can inject markup.
 */
export function Markdown({ text }: { text: string }) {
  return <div className="ew-md">{blocks(text)}</div>;
}

type Block =
  | { kind: "p"; lines: string[] }
  | { kind: "h"; level: number; text: string }
  | { kind: "ul" | "ol"; items: string[]; start: number }
  | { kind: "code"; text: string };

const BULLET = /^\s*[-*•]\s+(.*)$/;
const NUMBERED = /^\s*(\d{1,3})[.)]\s+(.*)$/;
const HEADING = /^(#{1,4})\s+(.*)$/;

export function parse(text: string): Block[] {
  const out: Block[] = [];
  const lines = text.replace(/\r\n/g, "\n").split("\n");
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const last = out[out.length - 1];
    if (line.trim().startsWith("```")) {
      const body: string[] = [];
      for (i++; i < lines.length && !lines[i].trim().startsWith("```"); i++) body.push(lines[i]);
      out.push({ kind: "code", text: body.join("\n") });
      continue;
    }
    if (!line.trim()) {
      out.push({ kind: "p", lines: [] });
      continue;
    }
    const h = HEADING.exec(line.trim());
    const b = BULLET.exec(line);
    const n = NUMBERED.exec(line);
    if (h) out.push({ kind: "h", level: h[1].length, text: h[2] });
    else if (b) {
      if (last?.kind === "ul") last.items.push(b[1]);
      else out.push({ kind: "ul", items: [b[1]], start: 1 });
    } else if (n) {
      if (last?.kind === "ol") last.items.push(n[2]);
      else out.push({ kind: "ol", items: [n[2]], start: Number(n[1]) });
    } else if ((last?.kind === "ul" || last?.kind === "ol") && /^\s{2,}\S/.test(line)) {
      // An indented line continues the previous list item.
      last.items[last.items.length - 1] += " " + line.trim();
    } else if (last?.kind === "p") last.lines.push(line);
    else out.push({ kind: "p", lines: [line] });
  }
  return out.filter((b) => b.kind !== "p" || b.lines.length > 0);
}

function blocks(text: string): ReactNode[] {
  return parse(text).map((b, i) => {
    switch (b.kind) {
      case "h":
        return b.level <= 2 ? <h3 key={i}>{inline(b.text)}</h3> : <h4 key={i}>{inline(b.text)}</h4>;
      case "ul":
        return <ul key={i}>{b.items.map((it, j) => <li key={j}>{inline(it)}</li>)}</ul>;
      case "ol":
        return <ol key={i} start={b.start}>{b.items.map((it, j) => <li key={j}>{inline(it)}</li>)}</ol>;
      case "code":
        return <pre key={i}><code>{b.text}</code></pre>;
      default:
        return (
          <p key={i}>
            {b.lines.map((l, j) => (
              <Fragment key={j}>
                {j > 0 && <br />}
                {inline(l)}
              </Fragment>
            ))}
          </p>
        );
    }
  });
}

const INLINE = /(\*\*[^*\n]+?\*\*|(?<!\w)__[^_\n]+?__(?!\w)|`[^`\n]+`|(?<![\w*])\*[^*\s][^*\n]*?\*(?![\w*])|(?<!\w)_[^_\s][^_\n]*?_(?!\w))/g;

export function inline(text: string): ReactNode[] {
  return text.split(INLINE).map((part, i) => {
    if (/^(\*\*|__).+(\*\*|__)$/.test(part)) return <strong key={i}>{part.slice(2, -2)}</strong>;
    if (/^`.+`$/.test(part)) return <code key={i}>{part.slice(1, -1)}</code>;
    // Single * or _ only counts as emphasis around a word, so "5 * 3" and
    // snake_case_names stay as written.
    if (/^\*[^*].*\*$/.test(part) || /^_[^_].*_$/.test(part)) return <em key={i}>{part.slice(1, -1)}</em>;
    return part;
  });
}
