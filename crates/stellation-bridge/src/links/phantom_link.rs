use std::marker::PhantomData;

use super::Link;

/// A Link that does nothing.
///
/// This is used as a type parameter for types that may or may not have a link.
#[derive(Debug, Clone)]
pub struct PhantomLink {
    _marker: PhantomData<()>,
}

impl PartialEq for PhantomLink {
    fn eq(&self, _other: &Self) -> bool {
        true
    }
}

impl Link for PhantomLink {
    fn with_token<T>(&self, _token: T) -> Self
    where
        T: AsRef<str>,
    {
        todo!()
    }

    fn resolve_encoded<'life0, 'life1, 'async_trait>(
        &'life0 self,
        _input_buf: &'life1 [u8],
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = crate::BridgeResult<Vec<u8>>> + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn resolve_mutation<'life0, 'life1, 'async_trait, T>(
        &'life0 self,
        _input: &'life1 T::Input,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = crate::routines::MutationResult<T>> + 'async_trait>,
    >
    where
        T: 'static + crate::routines::BridgedMutation,
        T: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }

    fn resolve_query<'life0, 'life1, 'async_trait, T>(
        &'life0 self,
        _input: &'life1 T::Input,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = crate::routines::QueryResult<T>> + 'async_trait>,
    >
    where
        T: 'static + crate::routines::BridgedQuery,
        T: 'async_trait,
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        todo!()
    }
}
