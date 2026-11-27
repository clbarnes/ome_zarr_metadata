//! Dimension-manipulating transformations: mapAxis and projectAxis.

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};
use validatrix::{Accumulator, DiagnosticError, Level, Validate};

use super::TransformType;

pub(super) fn unique_indices<'a>(
    accum: &mut Accumulator,
    key: &'static str,
    indices: impl IntoIterator<Item = &'a usize>,
) -> Result<BTreeSet<usize>, DiagnosticError> {
    let mut seen = BTreeSet::default();
    for (idx, i) in indices.into_iter().enumerate() {
        if !seen.insert(*i) {
            accum.with_keys(&[key.into(), idx.into()], |a| {
                a.diagnose(Level::Error, format_args!("duplicate index {i}"))
            })?;
        }
    }
    Ok(seen)
}

/// `mapAxis` transformation: an axis permutation.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct MapAxis {
    /// The value at position `i` is the index of the input axis which becomes the `i`th output axis.
    pub map_axis: Vec<usize>,
}

impl Validate for MapAxis {
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
        let n = self.map_axis.len();
        if n == 0 {
            accum.diagnose_at("mapAxis", Level::Error, "empty")?;
        }
        unique_indices(accum, "mapAxis", &self.map_axis)?;
        for (idx, i) in self.map_axis.iter().enumerate() {
            if *i >= n {
                accum.with_keys(&["mapAxis".into(), idx.into()], |a| {
                    a.diagnose(
                        Level::Error,
                        format_args!("index {i} out of range for {n} axes"),
                    )
                })?;
            }
        }
        Ok(())
    }
}

impl TransformType for MapAxis {
    fn input_ndim(&self) -> Option<usize> {
        Some(self.map_axis.len())
    }
}

/// `projectAxis` transformation: add and/or drop dimensions.
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ProjectAxis {
    /// Indices of input dimensions to drop.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dropped_inputs: Vec<usize>,
    /// Indices of output dimensions which are newly created (with value 0).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub created_outputs: Vec<usize>,
}

impl Validate for ProjectAxis {
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
        if self.dropped_inputs.is_empty() && self.created_outputs.is_empty() {
            accum.diagnose(
                Level::Error,
                "neither droppedInputs nor createdOutputs given",
            )?;
        }
        unique_indices(accum, "droppedInputs", &self.dropped_inputs)?;
        unique_indices(accum, "createdOutputs", &self.created_outputs)?;
        Ok(())
    }
}

impl TransformType for ProjectAxis {
    fn input_ndim(&self) -> Option<usize> {
        None
    }

    fn output_ndim(&self) -> Option<usize> {
        None
    }

    fn check_ndim(&self, input: usize, output: usize) -> Option<bool> {
        if let Some(di) = self.dropped_inputs.iter().max()
            && *di >= input
        {
            return Some(false);
        }
        if let Some(co) = self.created_outputs.iter().max()
            && *co >= output
        {
            return Some(false);
        }
        Some(true)
    }
}
