import { useState } from "react";
import { AlertTriangle, ArrowUpRight, Check, FileDown, FileText, LayoutGrid, ShieldCheck } from "lucide-react";
import { api } from "../api";
import type { DocumentsCard, ResultCard, SpreadsheetCard } from "../types";

// The design's report colours, extended in the same family.
const COLORS = ["#7e9065", "#b0bd99", "#d1c7a9", "#dfe3d4", "#94a27f", "#c4cfb2", "#b9ad8c", "#e9ece0"];
const money = (n: number) => n.toLocaleString(undefined, { minimumFractionDigits: 0, maximumFractionDigits: 2 });

/** Saves a reply (a result card or a written answer) as a formatted PDF. */
export function SavePdf({ messageId, onError }: { messageId: number; onError: (e: unknown) => void }) {
  const [saved, setSaved] = useState(false);
  return (
    <button className="ew-followup" onClick={() => api.exportPdf(messageId).then((ok) => ok && setSaved(true), onError)}>
      {saved ? <Check size={13} /> : <FileDown size={13} />}
      {saved ? "PDF saved" : "Save as PDF"}
    </button>
  );
}

/** A result card from the document or spreadsheet agent, with Save. */
export function ResultCardView({ card, messageId, onError }: { card: ResultCard; messageId: number; onError: (e: unknown) => void }) {
  const [saved, setSaved] = useState(false);
  const save = () =>
    api.exportCard(messageId).then((ok) => ok && setSaved(true), onError);
  return (
    <>
      {card.type === "documents" ? <Documents card={card} /> : <Spreadsheet card={card} />}
      <div className="ew-actions">
        <button className="ew-followup" onClick={save}>
          {saved ? <Check size={13} /> : null}
          {saved ? "Saved" : card.type === "documents" ? "Save as Markdown" : "Save as Excel report"}
          <ArrowUpRight size={14} />
        </button>
        <SavePdf messageId={messageId} onError={onError} />
      </div>
    </>
  );
}

function Documents({ card }: { card: DocumentsCard }) {
  const flagged = card.documents.some((d) => d.flagged);
  return (
    <div className="ew-report">
      <div className="ew-report-heading">
        <div>
          <span className="ew-report-icon"><FileText size={19} /></span>
          <strong>
            {card.documents.length === 1 ? card.documents[0].name : `${card.documents.length} documents`}
            <small>Summarized on this Mac · {card.folder}</small>
          </strong>
        </div>
        <span className="ew-ready ok"><Check size={11} />Ready</span>
      </div>
      {card.overview && <p className="ew-doc-overview">{card.overview}</p>}
      {card.documents.map((d) => (
        <details key={d.name} className="ew-doc" open={card.documents.length === 1}>
          <summary>
            <strong>{d.name}</strong>
            {d.flagged && <span className="ew-doc-flag" title="Contained instructions aimed at AI; ignored">⚠︎</span>}
          </summary>
          <p>{d.summary}</p>
          {d.keyPoints.length > 0 && (
            <ul>
              {d.keyPoints.map((k, i) => (
                <li key={i}>{k}</li>
              ))}
            </ul>
          )}
          {d.truncated && <p className="ew-doc-note">Long document: only the first part was summarized.</p>}
        </details>
      ))}
      {card.unreadable.length > 0 && (
        <p className="ew-doc-note ew-doc-unreadable">
          Couldn’t read: {card.unreadable.map(([n, why]) => `${n} (${why})`).join("; ")}
        </p>
      )}
      <div className="ew-report-foot">
        {flagged ? <AlertTriangle size={13} /> : <ShieldCheck size={13} />}
        {flagged
          ? "A document contained instructions aimed at AI assistants. Errandly removed and ignored them; double-check anything surprising."
          : "Read locally. Nothing was uploaded or changed."}
      </div>
    </div>
  );
}

function Spreadsheet({ card }: { card: SpreadsheetCard }) {
  const a = card.analysis;
  const max = Math.max(...a.groups.map((g) => Math.abs(g.total)), 1);
  return (
    <>
      <div className="ew-report">
        <div className="ew-report-heading">
          <div>
            <span className="ew-report-icon"><LayoutGrid size={19} /></span>
            <strong>
              {card.title}
              <small>
                {card.file}
                {card.sheet ? ` · ${card.sheet}` : ""} · {a.rows} rows
              </small>
            </strong>
          </div>
          <span className="ew-ready ok"><Check size={11} />Calculated</span>
        </div>
        {a.total !== null && (
          <div className="ew-total">
            <span>
              Total {a.valueColumn}
              <strong>{money(a.total)}</strong>
            </span>
            {a.groups.length > 0 && (
              <div className="ew-mini-chart" aria-hidden>
                {a.groups.map((g, i) => (
                  <i key={g.name} style={{ height: `${Math.max(10, (Math.abs(g.total) / max) * 100)}%`, background: COLORS[i % COLORS.length] }} />
                ))}
              </div>
            )}
          </div>
        )}
        {a.groups.length > 0 && (
          <>
            <div className="ew-category-bar ew-category-bar-live">
              {a.groups.map((g, i) => (
                <i key={g.name} style={{ width: `${Math.max(0, g.share)}%`, background: COLORS[i % COLORS.length] }} />
              ))}
            </div>
            <div className="ew-categories">
              {a.groups.map((g, i) => (
                <div key={g.name}>
                  <i style={{ background: COLORS[i % COLORS.length] }} />
                  <span>{g.name}</span>
                  <strong>{money(g.total)}</strong>
                  <small>{Math.round(g.share)}%</small>
                </div>
              ))}
            </div>
          </>
        )}
        {a.total === null && a.numeric.length === 0 && <p className="ew-doc-overview">No numeric columns found in this sheet.</p>}
        <div className="ew-report-foot">
          <ShieldCheck size={13} />
          Every number was calculated from the file{a.groupColumn ? `, grouped by ${a.groupColumn}` : ""}
          {a.skippedRows > 0 ? ` · ${a.skippedRows} row(s) without a number were left out` : ""}
        </div>
      </div>
      {card.insights.map((s, i) => (
        <p key={i} className="ew-takeaway">
          <span>↳</span> {s}
        </p>
      ))}
    </>
  );
}
