use crate::{
    scope::ScopePointer,
    r#type::Type,
    type_expr::{ScopePortal, TypeExpr},
};
use std::collections::BTreeMap;

impl<T: Type> TypeExpr<T, ScopePortal<T>> {
    /// # Returns
    /// - [TypeExpr::Never] if `a` and `b` have nothing in common.
    /// - `None` if an uninferred variable prevents the intersection from being known.
    pub fn intersection(
        a: &Self,
        b: &Self,
        scope_a: &ScopePointer<T>,
        scope_b: &ScopePointer<T>,
    ) -> Option<(Self, ScopePointer<T>)> {
        match (a, b) {
            (Self::Any, b) => Some((b.clone(), ScopePointer::clone(scope_b))),
            (a, Self::Any) => Some((a.clone(), ScopePointer::clone(scope_a))),
            (Self::Never, _) => Some((Self::Never, ScopePointer::clone(scope_a))),
            (_, Self::Never) => Some((Self::Never, ScopePointer::clone(scope_b))),

            // Type Params
            (
                a @ Self::TypeParameter(local_param_a, ..),
                Self::TypeParameter(local_param_b, ..),
            ) => {
                let (_var_a, param_scope_a) = scope_a.lookup(local_param_a)?;
                let (_var_b, param_scope_b) = scope_b.lookup(local_param_b)?;
                // First check if the two variables reference the same var.
                if local_param_a == local_param_b && param_scope_a == param_scope_b {
                    return Some((a.clone(), ScopePointer::clone(scope_a)));
                }
                let (Some((inferred_a, scope_a)), Some((inferred_b, scope_b))) = (
                    scope_a.lookup_inferred(local_param_a),
                    scope_b.lookup_inferred(local_param_b),
                ) else {
                    return None;
                };
                Self::intersection(&inferred_a, &inferred_b, &scope_a, &scope_b)
            }
            (Self::TypeParameter(param, _infer), b) => {
                let (inferred_a, scope_a) = scope_a.lookup_inferred(param)?;
                Self::intersection(&inferred_a, b, &scope_a, scope_b)
            }
            (a, Self::TypeParameter(param, _infer)) => {
                let (inferred_b, scope_b) = scope_b.lookup_inferred(param)?;
                Self::intersection(a, &inferred_b, scope_a, &scope_b)
            }

            // Portals
            (Self::ScopePortal { expr, scope }, b) => {
                Self::intersection(expr, b, &scope.portal, scope_b)
            }
            (a, Self::ScopePortal { expr, scope }) => {
                Self::intersection(a, expr, scope_a, &scope.portal)
            }

            (Self::Intersection(a_a, a_b), b) => {
                let (intersection_a, intersection_a_scope) =
                    Self::intersection(a_a, a_b, scope_a, scope_a)?;
                Self::intersection(&intersection_a, b, &intersection_a_scope, scope_b)
            }
            (a, Self::Intersection(b_a, b_b)) => {
                let (intersection_b, intersection_b_scope) =
                    Self::intersection(b_a, b_b, scope_b, scope_b)?;
                Self::intersection(a, &intersection_b, scope_a, &intersection_b_scope)
            }
            (Self::Operation { a, b, operator }, b_expr) => {
                let a_normalized = a.normalize(scope_a);
                let b_normalized = b.normalize(scope_a);
                Self::intersection(
                    &T::operation(&a_normalized, operator, &b_normalized),
                    b_expr,
                    scope_a,
                    scope_b,
                )
            }
            (a_expr, Self::Operation { a, b, operator }) => {
                let a_normalized = a.normalize(scope_b);
                let b_normalized = b.normalize(scope_b);
                Self::intersection(
                    a_expr,
                    &T::operation(&a_normalized, operator, &b_normalized),
                    scope_a,
                    scope_b,
                )
            }

            (Self::Conditional(conditional), b) => {
                Self::intersection(&conditional.distribute(scope_a)?, b, scope_a, scope_b)
            }
            (a, Self::Conditional(conditional)) => {
                Self::intersection(a, &conditional.distribute(scope_b)?, scope_a, scope_b)
            }

            (Self::Type(a), Self::Type(b)) if a == b => {
                Some((Self::Type(a.clone()), ScopePointer::clone(scope_a)))
            }
            (Self::Constructor { inner, .. }, Self::Type(inst)) if inner == inst => {
                Some((a.clone(), ScopePointer::clone(scope_a)))
            }
            (Self::Type(inst), Self::Constructor { inner, .. }) if inner == inst => {
                Some((b.clone(), ScopePointer::clone(scope_b)))
            }
            (
                Self::Constructor {
                    inner: inner_a,
                    parameters: parameters_a,
                },
                Self::Constructor {
                    inner: inner_b,
                    parameters: parameters_b,
                },
            ) if inner_a == inner_b => {
                let mut intersected_params = BTreeMap::new();
                for ident in parameters_a.keys().chain(parameters_b.keys()) {
                    if intersected_params.contains_key(ident) {
                        continue;
                    }
                    let (intersected_param, intersected_scope) =
                        match (parameters_a.get(ident), parameters_b.get(ident)) {
                            (Some(pa), Some(pb)) => Self::intersection(pa, pb, scope_a, scope_b)?,
                            (Some(pa), None) => (pa.clone(), ScopePointer::clone(scope_a)),
                            (None, Some(pb)) => (pb.clone(), ScopePointer::clone(scope_b)),
                            (None, None) => unreachable!(),
                        };

                    intersected_params.insert(
                        ident.clone(),
                        TypeExpr::ScopePortal {
                            expr: Box::new(intersected_param),
                            scope: ScopePortal {
                                portal: intersected_scope,
                            },
                        },
                    );
                }
                Some((
                    Self::Constructor {
                        inner: inner_a.clone(),
                        parameters: intersected_params,
                    },
                    ScopePointer::clone(scope_a),
                ))
            }
            (Self::Constructor { .. }, Self::Constructor { .. }) => {
                Some((Self::Never, ScopePointer::clone(scope_a)))
            }

            (Self::Index { expr, index }, b) => {
                let (index_type, index_scope) = expr.index(index, scope_a, scope_a)?;
                Self::intersection(&index_type, b, &index_scope, scope_b)
            }
            (a, Self::Index { expr, index }) => {
                let (index_type, index_scope) = expr.index(index, scope_b, scope_b)?;
                Self::intersection(a, &index_type, scope_a, &index_scope)
            }

            (Self::KeyOf(expr), b) => {
                let (key_type, key_scope) = expr.keyof(scope_a)?;
                Self::intersection(&key_type, b, &key_scope, scope_b)
            }
            (a, Self::KeyOf(expr)) => {
                let (key_type, key_scope) = expr.keyof(scope_b)?;
                Self::intersection(a, &key_type, scope_a, &key_scope)
            }

            (Self::NodeSignature(_), _) | (_, Self::NodeSignature(_)) => {
                Some((Self::Never, ScopePointer::new_root()))
            }

            (Self::PortTypes(_), _) | (_, Self::PortTypes(_)) => {
                Some((Self::Never, ScopePointer::new_root()))
            }

            // @Todo: Test this
            // type G = Prettify<({ a: number } | { b: number }) & ({ c: string } | { d: boolean })>;
            (Self::Union(a, b), c) => {
                let (a_intersection, a_scope) = Self::intersection(a, c, scope_a, scope_b)?;
                let (b_intersection, b_scope) = Self::intersection(b, c, scope_a, scope_b)?;

                if a_scope == b_scope {
                    Some((
                        Self::Union(Box::new(a_intersection), Box::new(b_intersection)),
                        a_scope.clone(),
                    ))
                } else {
                    Some((
                        Self::Union(
                            Box::new(Self::ScopePortal {
                                expr: Box::new(a_intersection),
                                scope: ScopePortal { portal: a_scope },
                            }),
                            Box::new(Self::ScopePortal {
                                expr: Box::new(b_intersection),
                                scope: ScopePortal { portal: b_scope },
                            }),
                        ),
                        ScopePointer::new_root(),
                    ))
                }
            }
            (a, Self::Union(b, c)) => {
                let (b_intersection, b_scope) = Self::intersection(a, b, scope_a, scope_b)?;
                let (c_intersection, c_scope) = Self::intersection(a, c, scope_a, scope_b)?;

                if b_scope == c_scope {
                    Some((
                        Self::Union(Box::new(b_intersection), Box::new(c_intersection)),
                        b_scope.clone(),
                    ))
                } else {
                    Some((
                        Self::Union(
                            Box::new(Self::ScopePortal {
                                expr: Box::new(b_intersection),
                                scope: ScopePortal { portal: b_scope },
                            }),
                            Box::new(Self::ScopePortal {
                                expr: Box::new(c_intersection),
                                scope: ScopePortal { portal: c_scope },
                            }),
                        ),
                        ScopePointer::new_root(),
                    ))
                }
            }

            (Self::Type(_), _) => Some((Self::Never, ScopePointer::clone(scope_a))),
            (_, Self::Type(_)) => Some((Self::Never, ScopePointer::clone(scope_a))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse::expr;

    #[test]
    fn test_intersection() {
        let scope = ScopePointer::new_root();
        assert_eq!(
            expr("{a: Integer, b: String}"),
            TypeExpr::intersection(&expr("{a: Integer}"), &expr("{b: String}"), &scope, &scope)
                .unwrap()
                .0
                .normalize(&scope),
        );
    }
}
