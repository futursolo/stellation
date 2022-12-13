use std::sync::Arc;

#[derive(Debug, Default)]
pub struct BridgeMetadata<CTX = ()> {
    token: Option<Arc<str>>,
    context: Arc<CTX>,
}

impl<CTX> BridgeMetadata<CTX> {
    pub fn new() -> Self
    where
        CTX: Default,
    {
        BridgeMetadata::default()
    }

    pub fn with_token<S>(mut self, token: S) -> Self
    where
        S: AsRef<str>,
    {
        self.token = Some(token.as_ref().into());
        self
    }

    pub fn with_context<C>(self, context: C) -> BridgeMetadata<C> {
        BridgeMetadata {
            token: self.token,
            context: context.into(),
        }
    }

    pub fn context(&self) -> &CTX {
        self.context.as_ref()
    }

    pub fn token(&self) -> Option<&str> {
        self.token.as_deref()
    }

    pub(crate) fn duplicate(&self) -> Self {
        Self {
            token: self.token.clone(),
            context: self.context.clone(),
        }
    }
}
