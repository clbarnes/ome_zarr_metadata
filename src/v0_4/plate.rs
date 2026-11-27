//! "plate" metadata
//!
//! <https://ngff.openmicroscopy.org/0.4/#plate-md>.

use std::{
    collections::{BTreeSet, HashSet},
    num::NonZeroU64,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use validatrix::{Accumulator, DiagnosticError, Level, Validate};

/// `plate` metadata. For high-content screening datasets.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct Plate {
    /// The version of the multiscale metadata of the image.
    #[serde(default)]
    pub version: super::ConstrainedVersion,
    /// A list of JSON objects defining the acquisitions for a given plate to which wells can refer to
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acquisitions: Option<Vec<PlateAcquisition>>,
    /// A list of JSON objects defining the columns of the plate
    pub columns: Vec<PlateColumn>,
    /// The field count defining the maximum number of fields per view across all wells (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field_count: Option<NonZeroU64>,
    /// The name of the plate (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Defines the rows of the plate.
    pub rows: Vec<PlateRow>,
    /// Defines the wells of the plate.
    pub wells: Vec<PlateWell>,
}

impl Validate for Plate {
    fn validate_inner(&self, accum: &mut validatrix::Accumulator) -> Result<(), DiagnosticError> {
        if let Some(acqs) = self.acquisitions.as_deref() {
            accum.with_key("acquisitions", |acc| {
                let mut visited = HashSet::with_capacity(acqs.len());
                for (idx, a) in acqs.iter().enumerate() {
                    if !visited.insert(a.id) {
                        acc.with_keys(&[idx.into(), "id".into()], |acc_inner| {
                            acc_inner
                                .diagnose(Level::Error, format_args!("not unique: {}", a.id))?;
                            Ok(())
                        })?;
                    }
                    acc.validate_member_at(idx, a)?;
                }
                Ok(())
            })?;
        }

        accum.with_key("columns", |acc| {
            let mut col_names = HashSet::with_capacity(self.columns.len());
            for (idx, col) in self.columns.iter().enumerate() {
                if !col_names.insert(&col.name) {
                    acc.with_keys(&[idx.into(), "name".into()], |acc_inner| {
                        acc_inner
                            .diagnose(Level::Error, format_args!("not unique: {}", col.name))?;
                        Ok(())
                    })?;
                }
                acc.validate_member_at(idx, col)?;
            }
            Ok(())
        })?;

        accum.with_key("rows", |acc| {
            let mut row_names = HashSet::with_capacity(self.rows.len());

            for (idx, row) in self.rows.iter().enumerate() {
                if !row_names.insert(&row.name) {
                    acc.with_keys(&[idx.into(), "name".into()], |acc_inner| {
                        acc_inner
                            .diagnose(Level::Error, format_args!("not unique: {}", row.name))?;
                        Ok(())
                    })?;
                }
                acc.validate_member_at(idx, row)?;
            }
            Ok(())
        })?;

        accum.with_key("wells", |acc| {
            let mut plate_well_paths: BTreeSet<&Path> = Default::default();
            for (idx, well) in self.wells.iter().enumerate() {
                acc.with_key(idx, |w_acc| {
                    if !plate_well_paths.insert(&well.path) {
                        w_acc.with_key("path", |a| {
                            a.diagnose(
                                Level::Error,
                                format_args!("duplicate well path: {}", well.path.display()),
                            )?;
                            Ok(())
                        })?;
                    }
                    let col_name_opt = self.columns.get(well.column_index).map(|c| c.name.as_str());
                    if col_name_opt.is_none() {
                        w_acc.diagnose_at(
                            "columnIndex",
                            Level::Error,
                            format_args!("column index {} does not exist", well.column_index),
                        )?;
                    }
                    let row_name_opt = self.rows.get(well.row_index).map(|r| r.name.as_str());
                    if row_name_opt.is_none() {
                        w_acc.diagnose_at(
                            "rowIndex",
                            Level::Error,
                            format_args!("row index {} does not exist", well.row_index),
                        )?;
                    }
                    if let (Some(col_name), Some(row_name)) = (col_name_opt, row_name_opt) {
                        let mut comp = well.path.components();
                        let Some(row_component) = comp.next() else {
                            w_acc.diagnose_at("path", Level::Error, "no row name")?;
                            return Ok(());
                        };

                        if row_component
                            .as_os_str()
                            .to_str()
                            .filter(|r| *r == row_name)
                            .is_none()
                        {
                            w_acc.diagnose_at(
                                "path",
                                Level::Error,
                                "row name does not match row index",
                            )?;
                        };

                        let Some(col_component) = comp.next() else {
                            w_acc.diagnose_at("path", Level::Error, "no column name")?;
                            return Ok(());
                        };

                        if col_component
                            .as_os_str()
                            .to_str()
                            .filter(|c| *c == col_name)
                            .is_none()
                        {
                            w_acc.diagnose_at(
                                "path",
                                Level::Error,
                                "column name does not match column index",
                            )?;
                        };

                        if comp.next().is_some() {
                            w_acc.diagnose_at("path", Level::Error, "too many path components")?;
                        }
                    }
                    Ok(())
                })?;
            }
            Ok(())
        })?;
        Ok(())
    }
}

/// [`Plate`] `acquisitions` element metadata. Defines a plate acquisition.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PlateAcquisition {
    /// A unique integer identifier that fields of view can refer to.
    pub id: u64,
    /// A string identifying the name of the acquisition (optional).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The maximum number of fields of view for the acquisition (optional).
    #[serde(skip_serializing_if = "Option::is_none", rename = "maximumfieldcount")]
    pub maximum_field_count: Option<NonZeroU64>,
    /// A string specifying a description for the acquisition (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// An epoch timestamp specifying the start timestamp of the acquisition (optional).
    #[serde(skip_serializing_if = "Option::is_none", rename = "starttime")]
    pub start_time: Option<u64>,
    /// An epoch timestamp specifying the end timestamp of the acquisition (optional).
    #[serde(skip_serializing_if = "Option::is_none", rename = "endtime")]
    pub end_time: Option<u64>,
}

impl Validate for PlateAcquisition {
    fn validate_inner(&self, accum: &mut validatrix::Accumulator) -> Result<(), DiagnosticError> {
        if let (Some(start), Some(end)) = (self.start_time, self.end_time)
            && end < start
        {
            accum.diagnose_at("endtime", Level::Error, "before starttime")?;
        }
        Ok(())
    }
}

/// [`Plate`] `columns` element metadata. Defines a plate column.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PlateColumn {
    /// Specifies the unique column mame.
    pub name: String,
}

impl Validate for PlateColumn {
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
        accum.with_key("name", |a| {
            validate_alphanum(a, &self.name)?;
            Ok(())
        })?;
        Ok(())
    }
}

/// [`Plate`] `rows` element metadata. Defines a plate row.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PlateRow {
    /// Specifies the unique row mame.
    pub name: String,
}

impl Validate for PlateRow {
    fn validate_inner(&self, accum: &mut Accumulator) -> Result<(), DiagnosticError> {
        accum.with_key("name", |a| {
            validate_alphanum(a, &self.name)?;
            Ok(())
        })?;
        Ok(())
    }
}

fn validate_alphanum(accum: &mut Accumulator, s: &str) -> Result<(), DiagnosticError> {
    for c in s.chars() {
        if !c.is_alphanumeric() {
            accum.diagnose(Level::Error, format_args!("not alphanumeric: {s}"))?;
            return Ok(());
        }
    }
    Ok(())
}

/// [`Plate`] `wells` element metadata. Defines a plate well.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct PlateWell {
    /// A string specifying the path to the well subgroup.
    pub path: PathBuf,
    /// Specifies the row index.
    #[serde(rename = "rowIndex")]
    pub row_index: usize,
    /// Specifies the column index.
    #[serde(rename = "columnIndex")]
    pub column_index: usize,
}

#[cfg(test)]
mod tests {
    use crate::v0_4::OmeNgffGroupAttributes;

    use super::*;

    #[test]
    fn plate_2wells() {
        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ome-zarr/specifications/0.4/examples/plate_strict/plate_2wells.json"
        ));
        let ome_metadata: OmeNgffGroupAttributes = serde_json::from_str(json).unwrap();
        let _plate: Plate = ome_metadata.plate.unwrap();
    }

    #[test]
    fn plate_6wells() {
        let json = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/ome-zarr/specifications/0.4/examples/plate_strict/plate_6wells.json"
        ));
        let ome_metadata: OmeNgffGroupAttributes = serde_json::from_str(json).unwrap();
        let _plate: Plate = ome_metadata.plate.unwrap();
    }
}
