//! This module provides the functionality to determine which nodes are suitable for connecting to a certain port.
//! It can be used to tell users which nodes they can use to connect to a certain port.
use crate::{
    nodety::inference::{Flow, Flows, InferenceConfig},
    scope::{Scope, ScopePointer},
    r#type::Type,
    type_expr::ScopedTypeExpr,
};

/// Determines whether or not a connection between an input and output port is valid.
pub fn is_compatible<T: Type>(
    output: &ScopedTypeExpr<T>,
    input: &ScopedTypeExpr<T>,
    output_scope: Scope<T>,
    input_scope: Scope<T>,
) -> bool {
    let output_scope_pointer = ScopePointer::new(output_scope);
    let input_scope_pointer = ScopePointer::new(input_scope);

    let flows = Flows {
        flows: vec![Flow {
            source: output,
            target: input,
            source_scope: ScopePointer::clone(&output_scope_pointer),
            target_scope: ScopePointer::clone(&input_scope_pointer),
        }],
    };

    flows.infer(InferenceConfig::default());

    output_scope_pointer.infer_defaults();
    input_scope_pointer.infer_defaults();

    input.supertype_of(output, &input_scope_pointer, &output_scope_pointer).is_supertype()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::notation::parse::{expr, scope};

    #[test]
    fn test_is_compatible() {
        assert!(is_compatible(&expr("Any"), &expr("Any"), scope("<>"), scope("<>")));

        assert!(is_compatible(&expr("Integer"), &expr("Any"), scope("<>"), scope("<>")));

        assert!(!is_compatible(&expr("Any"), &expr("Integer"), scope("<>"), scope("<>")));

        assert!(is_compatible(&expr("T"), &expr("T"), scope("<T>"), scope("<T>")));

        assert!(is_compatible(&expr("Array<T>"), &expr("T"), scope("<T>"), scope("<T>")));

        assert!(is_compatible(&expr("Array<T>"), &expr("Array<T>"), scope("<T>"), scope("<T>")));

        assert!(!is_compatible(
            &expr("T"),
            &expr("T"),
            scope("<T extends Array<Integer>>"),
            scope("<T extends Integer>"),
        ));
    }
}
