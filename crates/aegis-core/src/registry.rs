// AEGIS — zokastech.fr — Apache 2.0 / MIT

//! Thread-safe [`Recognizer`] registry and builder.
//!
//! ```
//! use aegis_core::registry::RecognizerRegistry;
//!
//! let reg = RecognizerRegistry::new();
//! assert!(reg.all().is_empty());
//! ```

use crate::entity::EntityType;
use crate::recognizer::Recognizer;
use std::sync::{Arc, RwLock};
use tracing::warn;

/// Registre thread-safe de recognizers.
#[derive(Default)]
pub struct RecognizerRegistry {
    inner: RwLock<Vec<Arc<dyn Recognizer>>>,
}

impl RecognizerRegistry {
    pub fn new() -> Self {
        Self {
            inner: RwLock::new(Vec::new()),
        }
    }

    fn write_vec(&self) -> std::sync::RwLockWriteGuard<'_, Vec<Arc<dyn Recognizer>>> {
        self.inner.write().unwrap_or_else(|e| {
            warn!(target: "aegis_core::registry", "RwLock poisoned (write), recovering");
            e.into_inner()
        })
    }

    fn read_vec(&self) -> std::sync::RwLockReadGuard<'_, Vec<Arc<dyn Recognizer>>> {
        self.inner.read().unwrap_or_else(|e| {
            warn!(target: "aegis_core::registry", "RwLock poisoned (read), recovering");
            e.into_inner()
        })
    }

    /// Enregistre un recognizer.
    pub fn add(&self, r: Arc<dyn Recognizer>) {
        tracing::trace!(target: "aegis_core::registry", name = %r.name(), "add recognizer");
        self.write_vec().push(r);
    }

    /// Removes the first recognizer with a matching name; returns `true` if one was removed.
    pub fn remove(&self, name: &str) -> bool {
        let mut g = self.write_vec();
        let before = g.len();
        g.retain(|x| x.name() != name);
        let removed = g.len() < before;
        if removed {
            tracing::trace!(target: "aegis_core::registry", %name, "remove recognizer");
        }
        removed
    }

    pub fn all(&self) -> Vec<Arc<dyn Recognizer>> {
        self.read_vec().clone()
    }

    pub fn get_by_entity(&self, et: &EntityType) -> Vec<Arc<dyn Recognizer>> {
        self.read_vec()
            .iter()
            .filter(|r| r.supported_entities().contains(et))
            .cloned()
            .collect()
    }

    pub fn get_by_language(&self, lang: &str) -> Vec<Arc<dyn Recognizer>> {
        let lang = lang.to_lowercase();
        self.read_vec()
            .iter()
            .filter(|r| {
                r.supported_languages()
                    .iter()
                    .any(|l| l.to_lowercase() == lang || *l == "*")
            })
            .cloned()
            .collect()
    }
}

/// Builder fluide pour construire un registre.
///
/// ```
/// # use aegis_core::{AnalysisConfig, EntityType, Recognizer, Entity};
/// # use aegis_core::registry::RecognizerRegistryBuilder;
/// # use std::sync::Arc;
/// struct R(&'static str);
/// impl Recognizer for R {
///     fn name(&self) -> &str { self.0 }
///     fn supported_entities(&self) -> Vec<EntityType> { vec![EntityType::Email] }
///     fn supported_languages(&self) -> Vec<&str> { vec!["fr"] }
///     fn analyze(&self, _: &str, _: &AnalysisConfig) -> Vec<Entity> { vec![] }
/// }
/// let reg = RecognizerRegistryBuilder::new()
///     .with(Arc::new(R("a")))
///     .with_recognizers([Arc::new(R("b")) as Arc<dyn Recognizer>])
///     .build();
/// assert_eq!(reg.all().len(), 2);
/// ```
pub struct RecognizerRegistryBuilder {
    reg: RecognizerRegistry,
}

impl RecognizerRegistryBuilder {
    pub fn new() -> Self {
        Self {
            reg: RecognizerRegistry::new(),
        }
    }

    pub fn with(self, r: Arc<dyn Recognizer>) -> Self {
        self.reg.add(r);
        self
    }

    pub fn with_recognizers<I>(self, iter: I) -> Self
    where
        I: IntoIterator<Item = Arc<dyn Recognizer>>,
    {
        for r in iter {
            self.reg.add(r);
        }
        self
    }

    pub fn build(self) -> RecognizerRegistry {
        self.reg
    }
}

impl Default for RecognizerRegistryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
