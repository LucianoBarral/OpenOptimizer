// Lê o CHANGELOG.md (Keep a Changelog) — a mesma fonte das notas da Release no GitHub.
// Só o subconjunto que usamos: "## [x.y.z] - data", "### Seção", "- item", **negrito**, `código`.
// Nada vira HTML: a UI renderiza segmentos como texto.

export type Section = { title: string; items: string[] };
export type Release = { version: string; date: string; sections: Section[] };
export type Inline = { kind: 'text' | 'bold' | 'code'; text: string };

const VERSION = /^##\s+\[([^\]]+)\](?:\s*-\s*(\S+))?/;

/** Corpo de uma versão (o que vem depois de "## [x]"; também o `body` do updater). */
export function parseSections(md: string): Section[] {
  const out: Section[] = [];
  let cur: Section | null = null;
  for (const raw of md.split(/\r?\n/)) {
    const line = raw.trim();
    if (line.startsWith('### ')) {
      cur = { title: line.slice(4).trim(), items: [] };
      out.push(cur);
    } else if (line.startsWith('- ') || line.startsWith('* ')) {
      if (!cur) {
        cur = { title: '', items: [] };
        out.push(cur);
      }
      cur.items.push(line.slice(2).trim());
    } else if (line && cur && cur.items.length && /^\s/.test(raw)) {
      // linha recuada = continuação do item anterior
      cur.items[cur.items.length - 1] += ' ' + line;
    }
  }
  return out.filter((s) => s.items.length);
}

export function parseChangelog(md: string): Release[] {
  const out: Release[] = [];
  let head: { version: string; date: string } | null = null;
  let body: string[] = [];
  const flush = () => {
    if (head) out.push({ ...head, sections: parseSections(body.join('\n')) });
  };
  for (const line of md.split(/\r?\n/)) {
    const m = VERSION.exec(line);
    if (m) {
      flush();
      head = { version: m[1], date: m[2] ?? '' };
      body = [];
    } else if (head && !/^\[[^\]]+\]:\s/.test(line)) {
      body.push(line);
    }
  }
  flush();
  return out;
}

export function parseInline(s: string): Inline[] {
  const out: Inline[] = [];
  const re = /\*\*(.+?)\*\*|`([^`]+)`/g;
  let last = 0;
  for (let m = re.exec(s); m; m = re.exec(s)) {
    if (m.index > last) out.push({ kind: 'text', text: s.slice(last, m.index) });
    out.push(m[1] !== undefined ? { kind: 'bold', text: m[1] } : { kind: 'code', text: m[2] });
    last = re.lastIndex;
  }
  if (last < s.length) out.push({ kind: 'text', text: s.slice(last) });
  return out;
}

/** Marcador entre as partes PT e EN no corpo da Release (invisível no GitHub). */
export const EN_MARKER = '<!-- en -->';

/** Corpo bilíngue da Release → só a parte do idioma. Corpos antigos (sem marcador) valem para os dois. */
export function pickLang(body: string, locale: string): string {
  const i = body.indexOf(EN_MARKER);
  if (i < 0) return body;
  return locale === 'en' ? body.slice(i + EN_MARKER.length) : body.slice(0, i);
}

/** "0.2.1" > "0.2.0"? Comparação numérica por partes (sem pré-release). */
export function isNewer(a: string, b: string): boolean {
  const pa = a.split('.').map(Number);
  const pb = b.split('.').map(Number);
  for (let i = 0; i < Math.max(pa.length, pb.length); i++) {
    const d = (pa[i] ?? 0) - (pb[i] ?? 0);
    if (d) return d > 0;
  }
  return false;
}
