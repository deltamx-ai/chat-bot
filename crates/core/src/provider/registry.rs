use super::ProviderDescriptor;

#[derive(Debug, Clone, Default)]
pub struct ProviderRegistry {
    providers: Vec<ProviderDescriptor>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, provider: ProviderDescriptor) {
        self.providers.push(provider);
    }

    pub fn all(&self) -> &[ProviderDescriptor] {
        &self.providers
    }
}
