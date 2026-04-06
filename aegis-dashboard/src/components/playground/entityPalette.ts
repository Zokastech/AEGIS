// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Tailwind classes to highlight an entity by type.
 * Product spec: PERSON=blue, EMAIL=green, IBAN=orange.
 */
/** Ring inset = no extra inline width (avoids wrap drift vs plain monospace). */
export const ENTITY_HIGHLIGHT_CLASSES: Record<string, string> = {
  PERSON: "bg-blue-500/35 text-blue-50 ring-1 ring-inset ring-blue-400/55",
  EMAIL: "bg-emerald-500/35 text-emerald-50 ring-1 ring-inset ring-emerald-400/55",
  IBAN: "bg-orange-500/35 text-orange-50 ring-1 ring-inset ring-orange-400/55",
  PHONE: "bg-teal-500/30 text-teal-50 ring-1 ring-inset ring-teal-400/50",
  CREDIT_CARD: "bg-amber-500/35 text-amber-50 ring-1 ring-inset ring-amber-400/55",
  IP_ADDRESS: "bg-cyan-500/30 text-cyan-50 ring-1 ring-inset ring-cyan-400/50",
  DATE_TIME: "bg-violet-500/30 text-violet-50 ring-1 ring-inset ring-violet-400/50",
  LOCATION: "bg-indigo-500/30 text-indigo-50 ring-1 ring-inset ring-indigo-400/50",
  DEFAULT: "bg-muted/80 text-foreground ring-1 ring-inset ring-border",
};

export function highlightClassForEntityType(entityType: string): string {
  return ENTITY_HIGHLIGHT_CLASSES[entityType] ?? ENTITY_HIGHLIGHT_CLASSES.DEFAULT;
}
