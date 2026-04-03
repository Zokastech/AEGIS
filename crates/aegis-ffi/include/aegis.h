/**
 * @file aegis.h
 * @brief API C stable du moteur AEGIS (zokastech.fr) — détection et anonymisation PII.
 *
 * Lier contre la bibliothèque `aegis_ffi` (`libaegis_ffi.so` / `libaegis_ffi.dylib` / `aegis_ffi.dll`).
 * Toutes les chaînes sont UTF-8. Les pointeurs retournés par les fonctions `aegis_*` (sauf
 * `aegis_last_error` et `aegis_version`) doivent être libérés avec `aegis_free_string` lorsqu’ils
 * sont non NULL.
 *
 * @copyright AEGIS — zokastech.fr — Apache 2.0 / MIT
 */

#ifndef AEGIS_H
#define AEGIS_H

#include <stddef.h>
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * Handle opaque du moteur (ne pas déréférencer).
 * Créé par aegis_init(), détruit par aegis_free().
 */
typedef struct AegisOpaque AegisHandle;

/**
 * @brief Initialise le moteur depuis une configuration JSON (schéma moteur AEGIS, équivalent YAML).
 *
 * @param config_json JSON UTF-8 terminé par NUL, ou NULL / chaîne vide pour la config par défaut
 *                    (recognizers regex `en` et `fr` si le crate regex est lié).
 * @return Handle à passer aux autres appels, ou NULL en cas d’erreur (voir aegis_last_error()).
 */
AegisHandle *aegis_init(const char *config_json);

/**
 * @brief Analyse un texte et retourne un JSON décrivant les entités détectées.
 *
 * @param handle Handle retourné par aegis_init().
 * @param text Texte UTF-8 terminé par NUL.
 * @param config_json Paramètres d’analyse partiels ou complets (JSON @c AnalysisConfig), ou NULL /
 *                    chaîne vide pour les défauts du moteur.
 * @return Chaîne JSON UTF-8 allouée par la librairie (à libérer avec aegis_free_string), ou NULL.
 */
char *aegis_analyze(AegisHandle *handle, const char *text, const char *config_json);

/**
 * @brief Analyse plusieurs textes (tableau JSON de chaînes).
 *
 * @param handle Handle moteur.
 * @param texts_json Tableau JSON, ex. `["ligne1","ligne2"]`, UTF-8 terminé par NUL.
 * @return Tableau JSON de résultats d’analyse ; à libérer avec aegis_free_string, ou NULL.
 */
char *aegis_analyze_batch(AegisHandle *handle, const char *texts_json);

/**
 * @brief Détecte les entités puis anonymise le texte selon la configuration des opérateurs.
 *
 * Le JSON retourné contient les champs `anonymized` et `analysis`.
 *
 * @param handle Handle moteur.
 * @param text Texte source UTF-8.
 * @param config_json JSON avec champs optionnels : analysis, operators_by_entity, default_operator.
 * @return JSON UTF-8 à libérer avec aegis_free_string, ou NULL.
 */
char *aegis_anonymize(AegisHandle *handle, const char *text, const char *config_json);

/**
 * @brief Libère une chaîne allouée par aegis_analyze, aegis_analyze_batch ou aegis_anonymize.
 * @param ptr Pointeur retourné par ces fonctions (NULL accepté, no-op).
 */
void aegis_free_string(char *ptr);

/**
 * @brief Détruit un handle créé par aegis_init.
 * @param handle Handle à libérer (NULL accepté).
 */
void aegis_free(AegisHandle *handle);

/**
 * @brief Dernier message d’erreur UTF-8 (statique ou géré par la lib), terminé par NUL.
 * Ne pas libérer. Chaîne vide si aucune erreur enregistrée.
 */
const char *aegis_last_error(void);

/**
 * @brief Version semver du composant FFI (ex. "0.1.0"). Ne pas libérer.
 */
const char *aegis_version(void);

#ifdef __cplusplus
}
#endif

#endif /* AEGIS_H */
