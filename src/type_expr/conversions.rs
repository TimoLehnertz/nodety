use crate::{
    scope::type_parameter::TypeParameter,
    r#type::Type,
    type_expr::{
        ErasedScopePortal, ScopePortal, TypeExpr, TypeExprScope, Unscoped,
        conditional::Conditional,
        node_signature::{NodeSignature, port_types::PortTypes},
    },
};

// Type expr conversions

impl<T: Type> From<TypeExpr<T, ScopePortal<T>>> for TypeExpr<T, ErasedScopePortal> {
    fn from(value: TypeExpr<T, ScopePortal<T>>) -> Self {
        value.map_scope_portals(&mut |_| ErasedScopePortal)
    }
}

impl<T: Type> From<TypeExpr<T, Unscoped>> for TypeExpr<T, ErasedScopePortal> {
    fn from(value: TypeExpr<T, Unscoped>) -> Self {
        value.map_scope_portals(&mut |_| ErasedScopePortal)
    }
}

impl<T: Type> From<TypeExpr<T, Unscoped>> for TypeExpr<T, ScopePortal<T>> {
    fn from(value: TypeExpr<T, Unscoped>) -> Self {
        value.map_scope_portals(&mut |_never| {
            unreachable!("Unscoped has no values and thus can never be constructed.")
        })
    }
}

// Type parameter conversions

impl<T: Type> From<TypeParameter<T, ScopePortal<T>>> for TypeParameter<T, ErasedScopePortal> {
    fn from(value: TypeParameter<T, ScopePortal<T>>) -> Self {
        value.map_scope_portals(&mut |_| ErasedScopePortal)
    }
}

impl<T: Type> From<TypeParameter<T, Unscoped>> for TypeParameter<T, ErasedScopePortal> {
    fn from(value: TypeParameter<T, Unscoped>) -> Self {
        value.map_scope_portals(&mut |_| ErasedScopePortal)
    }
}

impl<T: Type> From<TypeParameter<T, Unscoped>> for TypeParameter<T, ScopePortal<T>> {
    fn from(value: TypeParameter<T, Unscoped>) -> Self {
        value.map_scope_portals(&mut |_never| {
            unreachable!("Unscoped has no values and thus can never be constructed.")
        })
    }
}

// Node signature

impl<T: Type> From<NodeSignature<T, ScopePortal<T>>> for NodeSignature<T, ErasedScopePortal> {
    fn from(value: NodeSignature<T, ScopePortal<T>>) -> Self {
        value.map_scope_portals(&mut |_| ErasedScopePortal)
    }
}

impl<T: Type> From<NodeSignature<T, Unscoped>> for NodeSignature<T, ErasedScopePortal> {
    fn from(value: NodeSignature<T, Unscoped>) -> Self {
        value.map_scope_portals(&mut |_| ErasedScopePortal)
    }
}

impl<T: Type> From<NodeSignature<T, Unscoped>> for NodeSignature<T, ScopePortal<T>> {
    fn from(value: NodeSignature<T, Unscoped>) -> Self {
        value.map_scope_portals(&mut |_never| {
            unreachable!("Unscoped has no values and thus can never be constructed.")
        })
    }
}

impl<T: Type, S: TypeExprScope> TypeExpr<T, S> {
    pub fn map_scope_portals<SO: TypeExprScope>(
        self,
        mapper: &mut impl FnMut(S) -> SO,
    ) -> TypeExpr<T, SO> {
        match self {
            Self::Type(t) => TypeExpr::Type(t),
            Self::Constructor { inner, parameters } => TypeExpr::Constructor {
                inner,
                parameters: parameters
                    .into_iter()
                    .map(|(k, v)| (k, v.map_scope_portals(mapper)))
                    .collect(),
            },
            Self::Operation { a, operator, b } => TypeExpr::Operation {
                a: Box::new(a.map_scope_portals(mapper)),
                operator,
                b: Box::new(b.map_scope_portals(mapper)),
            },
            Self::TypeParameter(id, infer) => TypeExpr::TypeParameter(id, infer),
            Self::NodeSignature(sig) => {
                TypeExpr::NodeSignature(Box::new(sig.map_scope_portals(mapper)))
            }
            Self::PortTypes(pt) => TypeExpr::PortTypes(Box::new(pt.map_scope_portals(mapper))),
            Self::Union(a, b) => TypeExpr::Union(
                Box::new(a.map_scope_portals(mapper)),
                Box::new(b.map_scope_portals(mapper)),
            ),
            Self::KeyOf(expr) => TypeExpr::KeyOf(Box::new(expr.map_scope_portals(mapper))),
            Self::Index { expr, index } => TypeExpr::Index {
                expr: Box::new(expr.map_scope_portals(mapper)),
                index: Box::new(index.map_scope_portals(mapper)),
            },
            Self::Intersection(a, b) => TypeExpr::Intersection(
                Box::new(a.map_scope_portals(mapper)),
                Box::new(b.map_scope_portals(mapper)),
            ),
            Self::Conditional(conditional) => TypeExpr::Conditional(Box::new(Conditional {
                t_test: conditional.t_test.map_scope_portals(mapper),
                t_test_bound: conditional.t_test_bound.map_scope_portals(mapper),
                t_then: conditional.t_then.map_scope_portals(mapper),
                t_else: conditional.t_else.map_scope_portals(mapper),
                infer: conditional.infer,
            })),
            Self::Any => TypeExpr::Any,
            Self::Never => TypeExpr::Never,
            Self::ScopePortal { expr, scope } => TypeExpr::ScopePortal {
                expr: Box::new(expr.map_scope_portals(mapper)),
                scope: mapper(scope),
            },
        }
    }
}

impl<T: Type, S: TypeExprScope> PortTypes<T, S> {
    pub(crate) fn map_scope_portals<SO: TypeExprScope>(
        self,
        mapper: &mut impl FnMut(S) -> SO,
    ) -> PortTypes<T, SO> {
        PortTypes {
            ports: self
                .ports
                .into_iter()
                .map(|p| p.map_scope_portals(mapper))
                .collect(),
            varg: self.varg.map(|v| v.map_scope_portals(mapper)),
        }
    }
}

impl<T: Type, S: TypeExprScope> TypeParameter<T, S> {
    fn map_scope_portals<SO: TypeExprScope>(
        self,
        mapper: &mut impl FnMut(S) -> SO,
    ) -> TypeParameter<T, SO> {
        TypeParameter {
            bound: self.bound.map(|bound| bound.map_scope_portals(mapper)),
            default: self
                .default
                .map(|default| default.map_scope_portals(mapper)),
        }
    }
}

impl<T: Type, S: TypeExprScope> NodeSignature<T, S> {
    pub(crate) fn map_scope_portals<SO: TypeExprScope>(
        self,
        mapper: &mut impl FnMut(S) -> SO,
    ) -> NodeSignature<T, SO> {
        NodeSignature {
            parameters: self
                .parameters
                .into_iter()
                .map(|(k, param)| (k, param.map_scope_portals(mapper)))
                .collect(),
            inputs: self.inputs.map_scope_portals(mapper),
            outputs: self.outputs.map_scope_portals(mapper),
            default_input_types: self
                .default_input_types
                .into_iter()
                .map(|(k, v)| (k, v.map_scope_portals(mapper)))
                .collect(),
            tags: self.tags,
            required_tags: self.required_tags,
        }
    }
}
