use validatrix::{Accumulator, DiagnosticError, Level};

/// Trait for a type which has some dimensionality which can always be determined by its metadata.
pub trait NDim {
    /// Number of dimensions according to the metadata.
    fn ndim(&self) -> usize;
}

/// Trait for a type which has some dimensionality which may be determinable by its metadata.
pub trait MaybeNDim {
    /// None if number of dimensions is indeterminate from the metadata.
    fn maybe_ndim(&self) -> Option<usize>;

    /// If both objects have a dimensionality defined, but it's different,
    /// return Some with the two dimensionalities. Otherwise, return None.
    fn ndim_conflicts<T: MaybeNDim>(&self, other: &T) -> Option<(usize, usize)> {
        if let (Some(d1), Some(d2)) = (self.maybe_ndim(), other.maybe_ndim())
            && d1 != d2
        {
            return Some((d1, d2));
        }
        None
    }
}

impl<T: NDim> MaybeNDim for T {
    fn maybe_ndim(&self) -> Option<usize> {
        Some(NDim::ndim(self))
    }
}
pub(crate) fn validate_ndims<'a, T: MaybeNDim + 'a>(
    accum: &mut Accumulator,
    expected: Option<usize>,
    dimensionals: impl IntoIterator<Item = &'a T> + 'a,
) -> Result<(), DiagnosticError> {
    // iterator of idx, ndims
    let mut it = dimensionals
        .into_iter()
        .enumerate()
        .filter_map(|(idx, d)| d.maybe_ndim().map(|n| (idx, n)));
    let exp = match expected {
        Some(e) => e,
        None => {
            if let Some((_idx, n)) = it.next() {
                n
            } else {
                return Ok(());
            }
        }
    };
    for (idx, n) in it {
        if n != exp {
            accum.diagnose_at(
                idx,
                Level::Error,
                format_args!("inconsistent dimensionality: got {n}D, expected {exp}D"),
            )?;
        }
    }
    Ok(())
}
