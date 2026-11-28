import { describe, it, expect } from 'vitest';
import { parseChangelog, parseSections, parseInline } from './changelog';

const md = `# Changelog

Texto de introdução.

> Citação que não é versão.

## [0.2.1] - 2026-11-27

### Corrigido
- Barra de rolagem escura.
- Ícone **grande** da janela via \`WM_SETICON\`.

## [0.2.0] - 2026-11-26

### Adicionado
- **Modo Gaming** automático.
  continuação da mesma linha

### Segurança
- XML.

[0.2.1]: https://github.com/x/y/releases/tag/v0.2.1
`;

describe('changelog', () => {
  it('separa versões em ordem, com data, ignorando introdução e links', () => {
    const r = parseChangelog(md);
    expect(r.map((x) => x.version)).toEqual(['0.2.1', '0.2.0']);
    expect(r[0].date).toBe('2026-11-27');
    expect(r[1].sections.map((s) => s.title)).toEqual(['Adicionado', 'Segurança']);
  });

  it('junta linha de continuação ao item anterior', () => {
    const r = parseChangelog(md);
    expect(r[1].sections[0].items).toEqual(['**Modo Gaming** automático. continuação da mesma linha']);
  });

  it('lê o corpo da release (só seções, sem cabeçalho de versão)', () => {
    const s = parseSections('### Corrigido\n- A\n- B\n');
    expect(s).toEqual([{ title: 'Corrigido', items: ['A', 'B'] }]);
  });

  it('negrito e código viram segmentos, sem HTML', () => {
    expect(parseInline('Ícone **grande** via `WM_SETICON` <b>x</b>')).toEqual([
      { kind: 'text', text: 'Ícone ' },
      { kind: 'bold', text: 'grande' },
      { kind: 'text', text: ' via ' },
      { kind: 'code', text: 'WM_SETICON' },
      { kind: 'text', text: ' <b>x</b>' },
    ]);
  });
});
