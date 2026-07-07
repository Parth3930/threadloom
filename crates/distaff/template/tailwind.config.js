/** @type {import('tailwindcss').Config} */
module.exports = {
  darkMode: 'class',
  // Classes built dynamically by threadloom-ui from integer/string props (e.g. cols=2 → "grid-cols-2")
  // never appear as literal strings in .rs files, so Tailwind's scanner misses them. Safelist them here.
  safelist: [
    // Grid columns — base + responsive breakpoints
    ...[1,2,3,4,5,6,7,8,9,10,11,12].flatMap(n => [
      `grid-cols-${n}`, `sm:grid-cols-${n}`, `md:grid-cols-${n}`, `lg:grid-cols-${n}`
    ]),
    // Grid rows
    ...[1,2,3,4,5,6,7,8,9,10,11,12].flatMap(n => [
      `grid-rows-${n}`, `sm:grid-rows-${n}`, `md:grid-rows-${n}`, `lg:grid-rows-${n}`
    ]),
    // Gap
    ...[0,1,2,3,4,5,6,7,8,9,10,11,12,14,16,20,24,28,32,36,40,44,48,52,56,60,64,72,80,96].map(n => `gap-${n}`),
    // Spacing props (p, px, py, m, mx, my, mt, mb, ml, mr)
    ...['p','px','py','m','mx','my','mt','mb','ml','mr'].flatMap(p =>
      [1,2,3,4,5,6,8,12,16,20,24,32,40,48,52,56,60,64,72,80,96].map(n => `${p}-${n}`)
    ),
    // Flex utils
    'items-center','items-start','items-end','items-stretch','items-baseline',
    'justify-center','justify-start','justify-end','justify-between','justify-around','justify-evenly',
    // Misc layout
    'flex-wrap',
  ],

  content: {
    files: [
      "./src/**/*.rs", 
      "../threadloom/crates/**/*.rs",
      (process.env.HOME || process.env.USERPROFILE) + "/.cargo/git/checkouts/threadloom-*/**/*.rs",
      (process.env.HOME || process.env.USERPROFILE) + "/.cargo/registry/src/**/threadloom-*/**/*.rs",
      "./index.html"
    ],
    // Transform Rust source so Tailwind's JIT scanner can extract arbitrary-value classes
    // e.g. mt-[25rem], w-[220px], tracking-[0.2em] inside class="..." strings
    transform: {
      rs: (content) => {
        const matches = content.match(/class\s*=\s*"([^"]+)"/g) || [];
        // Emit as HTML class attributes so Tailwind's HTML parser picks up arbitrary values
        return matches
          .map(m => {
            const cls = m.replace(/^class\s*=\s*"/, '').replace(/"$/, '');
            return `<div class="${cls}"></div>`;
          })
          .join('\n');
      },
    },
  },
  theme: {
    extend: {
      spacing: {
        // Fill gaps in Tailwind's default scale
        "26": "6.5rem",
        "30": "7.5rem",
        "34": "8.5rem",
        "38": "9.5rem",
      },
      colors: {
        background: "var(--background)",
        foreground: "var(--foreground)",
        card: {
          DEFAULT: "var(--card)",
          foreground: "var(--card-foreground)",
        },
        popover: {
          DEFAULT: "var(--popover)",
          foreground: "var(--popover-foreground)",
        },
        primary: {
          DEFAULT: "var(--primary)",
          foreground: "var(--primary-foreground)",
        },
        secondary: {
          DEFAULT: "var(--secondary)",
          foreground: "var(--secondary-foreground)",
        },
        muted: {
          DEFAULT: "var(--muted)",
          foreground: "var(--muted-foreground)",
        },
        accent: {
          DEFAULT: "var(--accent)",
          foreground: "var(--accent-foreground)",
        },
        destructive: {
          DEFAULT: "var(--destructive)",
          foreground: "var(--destructive-foreground)",
        },
        border: "var(--border)",
        input: "var(--input)",
        ring: "var(--ring)",
      },
    },
  },
  plugins: [],
}
