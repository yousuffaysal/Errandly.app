import mark from "../assets/errandly-mark.png";

/**
 * Errandly's pillar mark. Drawn as a mask over the current text colour, so it
 * takes each spot's colour and size (1em) like the ✳ it replaces.
 */
export function Mark({ className = "" }: { className?: string }) {
  const url = `url(${mark})`;
  return <span className={`ew-mark-icon ${className}`} aria-hidden style={{ WebkitMaskImage: url, maskImage: url }} />;
}
