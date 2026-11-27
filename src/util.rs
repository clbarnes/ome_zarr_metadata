use std::collections::{BTreeSet, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

use validatrix::{Accumulator, DiagnosticError, Level, Validate};

#[derive(Debug, Clone, Copy)]
pub(crate) struct ValidNum(pub f64);

impl Validate for ValidNum {
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
        if self.0.is_nan() {
            accum.diagnose(Level::Error, "NaN is not allowed")?;
        } else if self.0.is_infinite() {
            accum.diagnose(Level::Error, "infinity is not allowed")?;
        }
        Ok(())
    }
}

#[allow(unused)]
pub fn unique_ord<T: Ord + Debug>(
    accum: &mut Accumulator,
    key: &'static str,
    values: impl IntoIterator<Item = T>,
    field_name: &'static str,
) -> Result<BTreeSet<T>, DiagnosticError> {
    let mut seen = BTreeSet::default();
    for (idx, v) in values.into_iter().enumerate() {
        if !seen.insert(v) {
            accum.with_keys(&[key.into(), idx.into()], move |a| {
                a.diagnose(
                    Level::Error,
                    format_args!("duplicate {field_name} at index {idx}"),
                )
            })?;
        }
    }
    Ok(seen)
}

#[allow(unused)]
pub fn unique_hash<T: Hash + Eq + Debug>(
    accum: &mut Accumulator,
    key: &'static str,
    values: impl IntoIterator<Item = T>,
    field_name: &'static str,
) -> Result<HashSet<T>, DiagnosticError> {
    let mut seen = HashSet::default();
    for (idx, v) in values.into_iter().enumerate() {
        if !seen.insert(v) {
            accum.with_keys(&[key.into(), idx.into()], |a| {
                a.diagnose(
                    Level::Error,
                    format_args!("duplicate {field_name} at index {idx}"),
                )
            })?;
        }
    }
    Ok(seen)
}

#[cfg(test)]
mod tests {
    use super::*;
    use validatrix::Validate;

    #[test]
    fn zero_allowed() {
        ValidNum(0.0).validate().unwrap();
    }
}
