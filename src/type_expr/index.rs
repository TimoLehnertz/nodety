use crate::{
    Type, TypeExpr,
    scope::ScopePointer,
    type_expr::{ScopePortal, ScopedTypeExpr},
};

impl<T: Type> TypeExpr<T, ScopePortal<T>> {
    /// Computes `self[index_type]`
    ///
    /// # Returns
    /// Some((indexed type, scope))
    ///
    /// If index_type is no legal index type for the type, returns Any.
    ///
    /// or None if
    /// - the index type is unknown due to uninferred vars.
    /// - Intersection or Union with distinct scopes.
    pub fn index(
        &self,
        index_type: &ScopedTypeExpr<T>,
        own_scope: &ScopePointer<T>,
        index_scope: &ScopePointer<T>,
    ) -> Option<(ScopedTypeExpr<T>, ScopePointer<T>)> {
        match self {
            Self::Type(inst) => {
                Some((inst.index(None, &index_type.normalize(index_scope)), ScopePointer::clone(own_scope)))
            }
            Self::Constructor { inner, parameters } => Some((
                inner.index(Some(parameters), &index_type.normalize(index_scope)),
                ScopePointer::clone(own_scope),
            )),

            // see tsReference.ts
            // @todo test this
            // Distributes over the union
            Self::Union(a, b) => {
                let (a_idx, a_scope) = a.index(index_type, own_scope, index_scope)?;
                let (b_idx, b_scope) = b.index(index_type, own_scope, index_scope)?;
                Some((
                    Self::Union(
                        Box::new(Self::ScopePortal { expr: Box::new(a_idx), scope: ScopePortal { portal: a_scope } }),
                        Box::new(Self::ScopePortal { expr: Box::new(b_idx), scope: ScopePortal { portal: b_scope } }),
                    ),
                    ScopePointer::clone(own_scope),
                ))
            }

            // see tsReference.ts
            // @todo test this
            // Distributes over the intersection
            Self::Intersection(a, b) => {
                let (a_idx, a_scope) = a.index(index_type, own_scope, index_scope)?;
                let (b_idx, b_scope) = b.index(index_type, own_scope, index_scope)?;

                Some((
                    Self::Intersection(
                        Box::new(Self::ScopePortal { expr: Box::new(a_idx), scope: ScopePortal { portal: a_scope } }),
                        Box::new(Self::ScopePortal { expr: Box::new(b_idx), scope: ScopePortal { portal: b_scope } }),
                    ),
                    ScopePointer::clone(own_scope),
                ))
            }

            Self::Operation { a, b, operator } => {
                let a_normalized = a.normalize(own_scope);
                let b_normalized = b.normalize(own_scope);
                T::operation(&a_normalized, operator, &b_normalized).index(index_type, own_scope, index_scope)
            }

            Self::TypeParameter(param, _infer) => {
                // Was:
                // if let Some((bound, scope)) = own_scope.lookup_bound(param) {
                // But in the case:      <T>                 <C>
                //                       | T['abc'] | ----- | C  |
                //
                // C will get inferred using the (bound of T)['abc'] Even when T is not yet inferred.
                if let Some((inferred, scope)) = own_scope.lookup_inferred(param) {
                    inferred.index(index_type, &scope, index_scope)
                } else {
                    None
                }
            }
            Self::ScopePortal { expr, scope } => expr.index(index_type, &scope.portal, index_scope),
            // These can't be indexed.
            Self::NodeSignature(_) => Some((Self::Any, ScopePointer::clone(own_scope))),
            Self::PortTypes(_) => Some((Self::Any, ScopePointer::clone(own_scope))),
            Self::Conditional { .. } => Some((Self::Any, ScopePointer::clone(own_scope))),
            Self::Any => Some((Self::Any, ScopePointer::clone(own_scope))),
            Self::Index { .. } => Some((Self::Any, ScopePointer::clone(own_scope))),
            Self::KeyOf(_) => Some((Self::Any, ScopePointer::clone(own_scope))),
            Self::Never => Some((Self::Any, ScopePointer::clone(own_scope))),
        }
    }
}
