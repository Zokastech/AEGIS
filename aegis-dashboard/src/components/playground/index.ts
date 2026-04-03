// AEGIS — zokastech.fr — Apache 2.0 / MIT

export type {
  PlaygroundEntity,
  DecisionTrace,
  DecisionTraceStep,
  AnonymizationLink,
  ApiEntityLike,
} from "./types";
export { playgroundEntityFromApi } from "./types";
export { ENTITY_HIGHLIGHT_CLASSES, highlightClassForEntityType } from "./entityPalette";
export { TextHighlighter, type TextHighlighterProps } from "./TextHighlighter";
export { EntitySidebar, type EntitySidebarProps } from "./EntitySidebar";
export { AnonymizationPreview, type AnonymizationPreviewProps } from "./AnonymizationPreview";
export { PipelineLevelSelector, type PipelineLevelSelectorProps, type PipelineLevel } from "./PipelineLevelSelector";
export { ConfidenceSlider, type ConfidenceSliderProps } from "./ConfidenceSlider";
export {
  AnonymizationOperatorSelect,
  type AnonymizationOperatorSelectProps,
} from "./AnonymizationOperatorSelect";
export { PlaygroundConfigPreview, type PlaygroundConfigPreviewProps } from "./PlaygroundConfigPreview";
