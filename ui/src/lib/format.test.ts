import { describe, it, expect } from 'vitest';
import { formatBytes, formatMs } from './format';

describe('format', () => {
  it('GB com 1 casa no separador do idioma', () => {
    expect(formatBytes(11.24 * 1024 ** 3, 'pt-BR')).toBe('11,2');
    expect(formatBytes(11.24 * 1024 ** 3, 'en')).toBe('11.2');
  });
  it('ms com 2 casas', () => expect(formatMs(15.625)).toBe('15.63'));
});
