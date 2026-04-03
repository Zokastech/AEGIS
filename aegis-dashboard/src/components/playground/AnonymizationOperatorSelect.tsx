// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useTranslation } from "react-i18next";
import { Label } from "@/components/ui/label";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import {
  PLAYGROUND_ANONYMIZE_OPERATOR_IDS,
  type PlaygroundAnonymizeOperatorId,
} from "@/lib/playground/anonymizeOperators";

export type AnonymizationOperatorSelectProps = {
  value: PlaygroundAnonymizeOperatorId;
  onChange: (v: PlaygroundAnonymizeOperatorId) => void;
  disabled?: boolean;
};

export function AnonymizationOperatorSelect({ value, onChange, disabled }: AnonymizationOperatorSelectProps) {
  const { t } = useTranslation("common");

  return (
    <div className="space-y-2">
      <Label className="text-foreground font-semibold">{t("playground.operator.label")}</Label>
      <p className="text-sm leading-relaxed text-foreground/82">{t("playground.operator.hint")}</p>
      <Select
        value={value}
        disabled={disabled}
        onValueChange={(v) => onChange(v as PlaygroundAnonymizeOperatorId)}
      >
        <SelectTrigger>
          <SelectValue />
        </SelectTrigger>
        <SelectContent>
          {PLAYGROUND_ANONYMIZE_OPERATOR_IDS.map((id) => (
            <SelectItem key={id} value={id}>
              {t(`playground.operator.${id}`)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
      <p className="text-sm leading-relaxed text-foreground/80">{t(`playground.operator.${value}Desc`)}</p>
      <p className="text-xs leading-snug text-muted-foreground">{t("playground.operator.footerNote")}</p>
      <p className="text-xs font-medium text-foreground/85">{t("playground.operator.rerunAnonymize")}</p>
    </div>
  );
}
