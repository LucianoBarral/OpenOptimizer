// Preferência de animação do app.
// O Windows com "Efeitos de animação" desligado (comum em PCs ajustados para desempenho)
// faz o WebView2 reportar prefers-reduced-motion: reduce. Antes o app desligava tudo por isso;
// agora o padrão é "Sempre", e "Seguir o Windows" fica disponível para quem precisa (acessibilidade).
import { writable, derived } from 'svelte/store';

export type MotionPref = 'always' | 'system';
const KEY = 'oo.motion';

function initial(): MotionPref {
  try {
    return localStorage.getItem(KEY) === 'system' ? 'system' : 'always';
  } catch {
    return 'always';
  }
}

const media = typeof matchMedia === 'function' ? matchMedia('(prefers-reduced-motion: reduce)') : null;
const systemReduce = writable(media?.matches ?? false);
media?.addEventListener('change', (e) => systemReduce.set(e.matches));

export const motionPref = writable<MotionPref>(initial());

export function setMotionPref(p: MotionPref) {
  motionPref.set(p);
  try {
    localStorage.setItem(KEY, p);
  } catch {
    /* ignora */
  }
}

/** true = sem animações (só quando o usuário escolheu seguir o Windows e o Windows pede menos movimento). */
export const reduceMotion = derived([motionPref, systemReduce], ([p, s]) => p === 'system' && s);

// Classe no <html> que o tokens.css usa para desligar animações.
reduceMotion.subscribe((r) => {
  if (typeof document !== 'undefined') document.documentElement.classList.toggle('reduce-motion', r);
});
