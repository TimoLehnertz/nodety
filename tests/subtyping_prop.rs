use nodety::{TypeExpr, demo_type::DemoType, scope::ScopePointer, type_expr::ScopePortal};
use proptest::prelude::*;

proptest! {
    #[test]
    #[ignore]
    fn test_a_supertype_of_a(expr in any::<TypeExpr<DemoType>>()) {
        let scoped: TypeExpr<DemoType, ScopePortal<DemoType>> = expr.into();

        let scope = ScopePointer::new_root();

        prop_assert!(scoped.supertype_of(&scoped, &scope, &scope).is_supertype());
    }
}
