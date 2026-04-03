// AEGIS — zokastech.fr — Apache 2.0 / MIT

/**
 * Tailwind classes to highlight an entity by type.
 * Product spec: PERSON=blue, EMAIL=green, IBAN=orange.
 */
export const ENTITY_HIGHLIGHT_CLASSES: Record<string, string> = {
  PERSON: "bg-blue-500/35 text-blue-50 border-blue-400/55",
  EMAIL: "bg-emerald-500/35 text-emerald-50 border-emerald-400/55",
  IBAN: "bg-orange-500/35 text-orange-50 border-orange-400/55",
  PHONE: "bg-teal-500/30 text-teal-50 border-teal-400/50",
  CREDIT_CARD: "bg-amber-500/35 text-amber-50 border-amber-400/55",
  IP_ADDRESS: "bg-cyan-500/30 text-cyan-50 border-cyan-400/50",
  DATE_TIME: "bg-violet-500/30 text-violet-50 border-violet-400/50",
  LOCATION: "bg-indigo-500/30 text-indigo-50 border-indigo-400/50",
  DEFAULT: "bg-muted/80 text-foreground border-border",
};

export function highlightClassForEntityType(entityType: string): string {
  return ENTITY_HIGHLIGHT_CLASSES[entityType] ?? ENTITY_HIGHLIGHT_CLASSES.DEFAULT;
}
