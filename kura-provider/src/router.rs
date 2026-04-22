use crate::adapter::*;
use kura_core::ProviderSpec;
use std::collections::HashMap;

pub struct ProviderRouter {
    providers: HashMap<String, Box<dyn ProviderAdapter>>,
    default: String,
}

impl ProviderRouter {
    pub fn new(specs: &[ProviderSpec]) -> Self {
        let mut providers: HashMap<String, Box<dyn ProviderAdapter>> = HashMap::new();
        let mut default_name = String::new();
        let mut best_priority = i64::MIN;

        for spec in specs {
            if spec.disabled {
                continue;
            }
            let adapter = crate::provider_from_spec(spec);
            if spec.priority > best_priority {
                best_priority = spec.priority;
                default_name = spec.name.clone();
            }
            providers.insert(spec.name.clone(), adapter);
        }

        if default_name.is_empty() {
            default_name = "zen".to_string();
        }

        Self {
            providers,
            default: default_name,
        }
    }

    pub fn get(&self, name: &str) -> Option<&dyn ProviderAdapter> {
        self.providers.get(name).map(|b| b.as_ref())
    }

    pub fn default_provider(&self) -> Option<&dyn ProviderAdapter> {
        self.get(&self.default)
    }

    pub fn default_name(&self) -> &str {
        &self.default
    }

    pub fn providers(&self) -> impl Iterator<Item = (&str, &dyn ProviderAdapter)> {
        self.providers.iter().map(|(k, v)| (k.as_str(), v.as_ref()))
    }
}
