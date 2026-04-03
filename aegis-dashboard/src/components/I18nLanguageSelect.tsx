// AEGIS — zokastech.fr — Apache 2.0 / MIT

import { useTranslation } from "react-i18next";
import { Languages } from "lucide-react";
import { cn } from "@/lib/utils";
import { supportedLanguages, type SupportedLanguage } from "@/i18n/config";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";

type I18nLanguageSelectProps = {
  /** Compact style for toolbars / mobile headers */
  readonly compact?: boolean;
  readonly className?: string;
};

function normalizeLng(lng: string | undefined): SupportedLanguage {
  const base = (lng ?? "en").split("-")[0] as SupportedLanguage;
  return supportedLanguages.includes(base) ? base : "en";
}

export function I18nLanguageSelect({ compact, className }: I18nLanguageSelectProps) {
  const { t, i18n } = useTranslation("common");
  const value = normalizeLng(i18n.resolvedLanguage || i18n.language);

  return (
    <div className={cn("flex items-center gap-2", className)}>
      {compact ? null : <Languages className="h-4 w-4 shrink-0 text-zokastech-gray" aria-hidden />}
      <Select
        value={value}
        onValueChange={(lng) => void i18n.changeLanguage(lng)}
      >
        <SelectTrigger
          className={cn(
            compact ? "h-8 w-[min(100%,9.5rem)] border-[#e2e8f0] bg-white text-xs" : "h-9 w-[11rem] border-[#e2e8f0] bg-white",
            "text-zokastech-dark"
          )}
          aria-label={t("language.label")}
        >
          <SelectValue placeholder={t("language.label")} />
        </SelectTrigger>
        <SelectContent align="end">
          {supportedLanguages.map((code) => (
            <SelectItem key={code} value={code}>
              {t(`language.${code}`)}
            </SelectItem>
          ))}
        </SelectContent>
      </Select>
    </div>
  );
}
