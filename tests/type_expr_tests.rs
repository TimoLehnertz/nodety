use maplit::hashset;
use nodety::{
    demo_type::DemoType,
    scope::{LocalParamID, Scope, ScopePointer, type_parameter::TypeParameter},
    type_expr::{TypeExpr, subtyping::SupertypeResult},
};

use crate::common::{expr, sig};

mod common;

#[test]
pub fn test_never_of_signatures() {
    let never_of_signatures = sig("Any -> Never");
    assert!(sig("(Integer) -> (String)").supertype_of(never_of_signatures.clone()).is_supertype());

    assert!(sig("<T>(T, String) -> (Integer, T)").supertype_of(never_of_signatures.clone()).is_supertype());
    assert!(!never_of_signatures.clone().supertype_of(sig("<T>(T, String) -> (Integer, T)")).is_supertype());

    assert!(never_of_signatures.clone().supertype_of(never_of_signatures.clone()).is_supertype());
}

#[test]
pub fn test_mother_of_signatures() {
    let mut mother_of_signatures = sig("Never -> Any");
    mother_of_signatures.tags = None;

    let mut child = sig("<T>(T, String) -> (Integer, T)");
    child.tags = Some(hashset! {123});

    assert!(mother_of_signatures.clone().supertype_of(child).is_supertype());

    assert!(mother_of_signatures.clone().supertype_of(mother_of_signatures).is_supertype());
}

#[test]
pub fn test_conditional_type() {
    let non_unit = expr("(String | Unit) extends Unit ? Never : String");

    let scope = ScopePointer::<DemoType>::new(Scope::new_root());

    assert_eq!(expr("String"), non_unit.normalize(&scope));
}

#[test]
pub fn test_conditional_type_2() {
    let non_unit = expr("'abc' extends String ? Integer : Unit");
    assert_eq!(expr("Integer"), non_unit.normalize_naive());
}

#[test]
pub fn test_conditional_type_generic() {
    let non_unit = expr("#0 extends Unit ? Never : #0");

    let mut scope = Scope::new_root();

    scope.define(LocalParamID(0), TypeParameter::default());

    let scope = ScopePointer::new(scope);

    scope.infer(&LocalParamID(0), expr("String|Unit"), ScopePointer::new_root()).unwrap();

    assert_eq!(expr("String"), non_unit.normalize(&scope));

    assert!(non_unit.supertype_of(&expr("String"), &scope, &scope).is_supertype());
    assert!(expr("String").supertype_of(&non_unit, &scope, &scope).is_supertype());
}

#[test]
fn test_generic_index_supertype() {
    let index_expr = expr("#0[Integer]");
    let mut scope = Scope::new_root();
    scope.define(LocalParamID(0), TypeParameter::default());
    scope.infer(&LocalParamID(0), expr("Array<Integer>"), ScopePointer::new_root()).unwrap();
    let scope = ScopePointer::new(scope);

    assert_eq!(index_expr.supertype_of(&index_expr, &scope, &scope), SupertypeResult::Supertype);
}

#[test]
fn test_generic_keyof_supertype() {
    let keyof_expr = expr("keyof #0");
    let mut scope = Scope::new_root();
    scope.define(LocalParamID(0), TypeParameter::default());
    scope.infer(&LocalParamID(0), expr("{a: Integer, b: String}"), ScopePointer::new_root()).unwrap();
    let scope = ScopePointer::new(scope);

    let normalized = keyof_expr.normalize(&scope);

    // Order of a and b is not deterministic because of hashmap.
    assert!(expr("'a'|'b'") == normalized || expr("'b'|'a'") == normalized);
}

#[test]
fn test_keyof_any() {
    let scope = ScopePointer::new_root();
    // Used to fail
    assert!(expr("keyof Any").supertype_of(&expr("keyof Any"), &scope, &scope).is_supertype());
}

#[test]
fn test_never_intersection_supertype() {
    let scope = ScopePointer::new_root();
    // Used to fail
    assert!(expr("Integer & Sortable").supertype_of(&expr("Integer & Sortable"), &scope, &scope).is_supertype());
}

#[test]
fn test_keyof_never_intersection() {
    let scope = ScopePointer::new_root();
    // The intersection of a record with Never is Never, so keyof should be Never too
    let normalized = expr("keyof ({a: Integer} & Never)").normalize(&scope);
    assert_eq!(expr("Never"), normalized);
}

#[test]
fn test_keyof_any_intersection() {
    let scope = ScopePointer::new_root();
    // Any & T = T, so keyof(Any & {a: Integer}) = keyof({a: Integer}) = 'a'
    let normalized = expr("keyof (Any & {a: Integer})").normalize(&scope);
    assert_eq!(normalized, expr("'a'"));
}

#[test]
fn test_keyof_union_with_never() {
    let scope = ScopePointer::new_root();
    // Never | T = T, so keyof(Never | {a: Integer}) = keyof({a: Integer}) = 'a'
    let normalized = expr("keyof (Never | {a: Integer})").normalize(&scope);
    assert_eq!(normalized, expr("'a'"));

    // Same when Never is on the right: keyof({a: Integer} | Never) = 'a'
    let normalized = expr("keyof ({a: Integer} | Never)").normalize(&scope);
    assert_eq!(normalized, expr("'a'"));
}

#[test]
fn test_union_supertypes() {
    assert!(expr("keyof {time: Duration | Unit, DNS: Boolean, DNF: Boolean, DQT: Boolean, false-starts: Integer, DQS: Boolean}").supertype_of_naive(&expr("'false-starts' | 'DQT'")).is_supertype());
}

#[test]
fn test_something() {
    let mut scope = Scope::new_root();

    scope.define(LocalParamID(0), TypeParameter::default());
    scope.define(LocalParamID(1), TypeParameter::default());

    scope.infer(&LocalParamID(0), expr("{a: Integer}"), ScopePointer::new_root()).unwrap();
    scope.infer(&LocalParamID(1), expr("{b: Float}"), ScopePointer::new_root()).unwrap();

    let scope = ScopePointer::new(scope);

    assert!(expr("{a: Integer, b: Float}").supertype_of(&expr("#0 & #1"), &scope, &scope).is_supertype());
    assert!(!expr("{a: Integer, c: Float}").supertype_of(&expr("#0 & #1"), &scope, &scope).is_supertype());
}

#[test]
fn test_infer_from() {
    let scope = ScopePointer::new_root();

    let source = expr("<#0>(#0) -> (#0)");
    let target = expr("<#1>(#1) -> (#1)");

    let (_, inferred_scope) = target.build_inferred_child_scope(&source, &scope, &scope);

    let (inferred, _inferred_scope) = inferred_scope.lookup_inferred(&LocalParamID(1)).unwrap();

    assert_eq!(TypeExpr::TypeParameter(LocalParamID(0), true), inferred);
}

#[test]
fn test_operation_supertypes() {
    assert!(expr("Any * Any").supertype_of_naive(&expr("Any * Any")).is_supertype());
}

#[test]
fn test_never_bounded_param_supertype_of_self() {
    let a = expr("<#0 extends Never>(#0) -> ()");
    assert!(a.supertype_of_naive(&a).is_supertype());
}
