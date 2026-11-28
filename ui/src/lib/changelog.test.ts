import { describe, it, expect } from 'vitest';
// Mesmo carregamento do app (?raw), sem depender de tipos do Node no tsconfig do app.
import rawPt from '../../../CHANGELOG.md?raw';
import rawEn from '../../../CHANGELOG.en.md?raw';
import { parseChangelog, parseSections, parseInline, pickLang } from './changelog';

describe('CHANGELOG.md × CHANGELOG.en.md', () => {
  const pt = parseChangelog(rawPt);
  const en = parseChangelog(rawEn);
  // Mesma estrutura nos dois idiomas: versões, datas e quantidade de itens por seção.
  const shape = (r: typeof pt) => r.map((x) => [x.version, x.date, x.sections.map((s) => s.items.length)]);
  it('têm as mesmas versões, datas e itens', () => expect(shape(en)).toEqual(shape(pt)));
  it('não estão vazios', () => expect(pt.length).toBeGreaterThan(0));
});

describe('pickLang (corpo bilíngue da Release)', () => {
  const body = '### Corrigido\n- A\n\n---\n<!-- en -->\n### Fixed\n- A (en)\n\n**Instalar:** x';
  it('pt-BR fica com a parte de cima', () => expect(parseSections(pickLang(body, 'pt-BR'))).toEqual([{ title: 'Corrigido', items: ['A'] }]));
  it('en fica com a parte de baixo', () => expect(parseSections(pickLang(body, 'en'))).toEqual([{ title: 'Fixed', items: ['A (en)'] }]));
  it('corpo antigo sem marcador vale para os dois', () => expect(pickLang('- A', 'en')).toBe('- A'));
});

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
