// Formatting and escaping, shared by everything that builds a string for the page.

const ESC = { '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;' };
export const escape = (s) => String(s).replace(/[&<>"]/g, (c) => ESC[c]);

export const fixed3 = (x) => x.toFixed(3);
export const commas = (n) => n.toLocaleString('en-US');

// `a` as a percentage of `b`, or of 1 when the share is already worked out
export const pct = (a, b = 1) => (b > 0 ? fixed3(100 * a / b) + '%' : '-');

export const hex = (b) => b.toString(16).padStart(2, '0');
