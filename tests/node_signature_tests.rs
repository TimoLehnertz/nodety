use crate::common::sig;

mod common;

#[test]
fn test_types_covariant() {
    let parent = sig("() -> (Any)");
    let child = sig("() -> (Integer)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_types_contravariant() {
    let parent = sig("(Integer) -> ()");
    let child = sig("(Any) -> ()");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_extra_argument() {
    let parent = sig("(Integer) -> ()");
    let child = sig("() -> ()");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_type_parameter_binding() {
    let parent = sig("() -> (Integer)");
    let child = sig("<A>() -> (A)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_identity() {
    let parent = sig("<#0>(#0) -> (#0)");
    let child = sig("<#1>(#1) -> (#1)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_identity_and_int_int() {
    let parent = sig("(Integer) -> (Integer)");
    let child = sig("<A>(A) -> (A)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_complex_type_parameter_substitution() {
    let parent = sig("(Integer) -> (Float)");
    let child = sig("<A, B>(A) -> (B)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_type_parameter_boundary_violation() {
    let parent = sig("() -> (Any)");
    let child = sig("<A extends Integer> () -> (A)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_output_varg_subtyping() {
    let parent = sig("() -> (Integer, Float, Float)");
    let child = sig("() -> (Integer, ...Float)");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_input_varg_subtyping() {
    let parent = sig("(Integer, ...Float) -> ()");
    let child = sig("(Integer, Float, Float) -> ()");

    assert!(!parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_input_varg_subtyping_2() {
    let parent = sig("(Integer, ...Integer) -> ()");
    let child = sig("(Integer) -> ()");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_input_varg_subtyping_3() {
    let parent = sig("(Integer, ...Integer) -> ()");
    let child = sig("(Integer) -> ()");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_node_sig_subtyping() {
    let parent = sig("(Integer) -> ()");
    let child = sig("() -> ()");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(!child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_node_sig_subtyping_both_generic() {
    let parent = sig("<#0 extends Integer>(#0) -> ()");
    let child = sig("<#1 extends Integer>(#1) -> ()");

    assert!(parent.clone().supertype_of(child.clone()).is_supertype());
    assert!(child.clone().supertype_of(parent.clone()).is_supertype());
}

#[test]
fn test_ignored_infer_param_still_gets_inferred_for_subtyping() {
    let parent = sig("<#0>(!#0) -> ()");
    assert!(parent.clone().supertype_of(parent.clone()).is_supertype());
}
