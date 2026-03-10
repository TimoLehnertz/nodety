use crate::{
    Type,
    scope::{LocalParamID, type_parameter::TypeParameter},
    type_expr::{TypeExprScope, Unscoped},
};
use std::{
    collections::BTreeMap,
    iter::FromIterator,
    ops::{Deref, DerefMut},
};

#[cfg(feature = "json-schema")]
use schemars::JsonSchema;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Wrapper for BtreeMap
///
/// Exists so that it can implement traits like [std::str::FromStr]
#[derive(Debug, PartialEq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(
    feature = "serde",
    serde(bound(
        serialize = "T: Serialize, T::Operator: Serialize, S: Serialize",
        deserialize = "T: Deserialize<'de>, T::Operator: Deserialize<'de>, S: Deserialize<'de>"
    ))
)]
#[cfg_attr(feature = "json-schema", derive(JsonSchema))]
#[cfg_attr(feature = "json-schema", schemars(bound = "T: JsonSchema, T::Operator: JsonSchema, S: JsonSchema"))]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct TypeParameters<T: Type, S: TypeExprScope = Unscoped>(pub BTreeMap<LocalParamID, TypeParameter<T, S>>);

impl<T: Type, S: TypeExprScope> Deref for TypeParameters<T, S> {
    type Target = BTreeMap<LocalParamID, TypeParameter<T, S>>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Type, S: TypeExprScope> DerefMut for TypeParameters<T, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Type, S: TypeExprScope> Default for TypeParameters<T, S> {
    fn default() -> Self {
        Self(BTreeMap::new())
    }
}

impl<T: Type, S: TypeExprScope> From<BTreeMap<LocalParamID, TypeParameter<T, S>>> for TypeParameters<T, S> {
    fn from(map: BTreeMap<LocalParamID, TypeParameter<T, S>>) -> Self {
        Self(map)
    }
}

impl<T: Type, S: TypeExprScope> FromIterator<(LocalParamID, TypeParameter<T, S>)> for TypeParameters<T, S> {
    fn from_iter<I: IntoIterator<Item = (LocalParamID, TypeParameter<T, S>)>>(iter: I) -> Self {
        Self(iter.into_iter().collect::<BTreeMap<_, _>>())
    }
}

impl<T: Type, S: TypeExprScope> IntoIterator for TypeParameters<T, S> {
    type Item = (LocalParamID, TypeParameter<T, S>);
    type IntoIter = std::collections::btree_map::IntoIter<LocalParamID, TypeParameter<T, S>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T: Type, S: TypeExprScope> IntoIterator for &'a TypeParameters<T, S> {
    type Item = (&'a LocalParamID, &'a TypeParameter<T, S>);
    type IntoIter = std::collections::btree_map::Iter<'a, LocalParamID, TypeParameter<T, S>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
