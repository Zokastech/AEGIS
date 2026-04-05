# AEGIS — zokastech.fr — Apache 2.0 / MIT

# Brand guidelines (Zokastech)

Visual and UX conventions for **AEGIS** product surfaces (dashboard, landing, docs) when aligned with the **Zokastech** identity.

## Color palette

| Token | Hex | Usage |
|-------|-----|--------|
| **Orange** | `#ff8a00` | Primary, CTAs, accents (`brand.orange` in Tailwind) |
| **Pink / magenta** | `#e52e71` | Secondary, privacy-style links, alternate hovers (`brand.pink`) |
| **Blue** | `#4361ee` | Third palette color, gradients, ribbons (`brand.blue`) |

**Signature gradient:** `linear-gradient(135deg, #ff8a00 → #e52e71 → #4361ee)`.

**CTA hover (e.g. start buttons):** lighter orange → pink gradient variant.

### Neutrals

- Page background: slate-50 (~`#f8fafc`) on `body`.
- Primary text: slate-900 / `--zokastech-dark` `#1a1d24`.
- Support gray: `--zokastech-gray` `#64748b`; form labels `#475569`; placeholders `#94a3b8`.

### Legacy / extension (Tailwind)

- `primary.200`: `#15151e` (blue-black).
- Secondary extension: violet / rose (`#912BBC`, `#D875C7`).

### Footer

- Background: `#1a1d24`, text white, links `#f1f5f9`, link hover orange `#ff8a00`.

### Forms

- Field background white, border `#e2e8f0`, focus border orange with halo `rgba(255, 138, 0, 0.2)`.

### Status

- Success: greens such as `#2f855a` with light background.
- Error: reds with visual tie to brand pink (e.g. pink-tinted border `#e52e71`).

### Shadows

- `shadow-brand` / `shadow-brand-lg`: orange-tinted `rgba(255, 138, 0, …)`.

## Typography

- **Font:** Plus Jakarta Sans (Google Fonts).
- Weights: 400, 500, 600, 700 + italics 400 and 500.
- Stack: `"Plus Jakarta Sans", system-ui, sans-serif`.
- `antialiased` on `body`.

## Layout

- Centered container: default horizontal padding **20px**, **40px** from `md` breakpoint.
- CSS variable: `--header-height: 4rem` where applicable.

## Motion

- `scroll-behavior: smooth` on `html`; respect `prefers-reduced-motion: reduce` (disable or minimize animations).
- Durations: **150ms**, **200ms**, **300ms**; easing `cubic-bezier(0.4, 0, 0.2, 1)`.
- Optional Tailwind-style animations: `fade-in` (8px), `slide-up` (16px), `gradient` (6s background) — neutered under reduced motion.

## UI patterns

- **Strong buttons:** orange → pink (→ blue) gradient, white text, hover slight `translateY(-2px)` and pink/orange shadow.
- **Hero / ribbons:** coherent with brand orange (e.g. `from-orange-500 to-orange-600` in Tailwind where used).
- **Cards:** white background, light gray borders, moderate shadow, rounded `lg` / `xl` / `2xl` by breakpoint.

## Language

- Default document language for public sites: **fr** where the product targets French audiences; technical docs remain multilingual via MkDocs.

## PDF / seal

- Signature seal artwork reuses orange, pink, blue on a very light background.

---

*This page summarizes design tokens for contributors; implementation lives in `aegis-dashboard` and `landing` CSS/Tailwind.*
