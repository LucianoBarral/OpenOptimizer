import { it, expect } from 'vitest';
import ptBR from './pt-BR';
import en from './en';

it('en e pt-BR têm exatamente as mesmas chaves', () => {
  expect(Object.keys(en).sort()).toEqual(Object.keys(ptBR).sort());
});
