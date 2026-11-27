/** Bytes → GB com 1 casa, no separador decimal do idioma. */
export const formatBytes = (n: number, locale: string) =>
  (n / 1024 ** 3).toLocaleString(locale, { minimumFractionDigits: 1, maximumFractionDigits: 1 });

/** Milissegundos com 2 casas. */
export const formatMs = (n: number) => n.toFixed(2);
