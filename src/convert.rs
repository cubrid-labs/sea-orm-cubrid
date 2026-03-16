//! Type conversion between SeaQuery values and CUBRID protocol values.

use cubrid_protocol::value::Value as CubridValue;
use sea_query::Value as SeaValue;

/// Convert SeaQuery values into CUBRID protocol values.
pub fn sea_value_to_cubrid(value: &SeaValue) -> CubridValue {
    match value {
        SeaValue::Bool(v) => v.map(CubridValue::Bool).unwrap_or(CubridValue::Null),
        SeaValue::TinyInt(v) => v
            .map(|v| CubridValue::Short(i16::from(v)))
            .unwrap_or(CubridValue::Null),
        SeaValue::SmallInt(v) => v.map(CubridValue::Short).unwrap_or(CubridValue::Null),
        SeaValue::Int(v) => v.map(CubridValue::Int).unwrap_or(CubridValue::Null),
        SeaValue::BigInt(v) => v.map(CubridValue::Long).unwrap_or(CubridValue::Null),
        SeaValue::TinyUnsigned(v) => v
            .map(|v| CubridValue::Int(i32::from(v)))
            .unwrap_or(CubridValue::Null),
        SeaValue::SmallUnsigned(v) => v
            .map(|v| CubridValue::Int(i32::from(v)))
            .unwrap_or(CubridValue::Null),
        SeaValue::Unsigned(v) => v
            .map(|v| match i32::try_from(v) {
                Ok(v) => CubridValue::Int(v),
                Err(_) => CubridValue::String(v.to_string()),
            })
            .unwrap_or(CubridValue::Null),
        SeaValue::BigUnsigned(v) => v
            .map(|v| match i64::try_from(v) {
                Ok(v) => CubridValue::Long(v),
                Err(_) => CubridValue::String(v.to_string()),
            })
            .unwrap_or(CubridValue::Null),
        SeaValue::Float(v) => v.map(CubridValue::Float).unwrap_or(CubridValue::Null),
        SeaValue::Double(v) => v.map(CubridValue::Double).unwrap_or(CubridValue::Null),
        SeaValue::String(v) => v
            .as_ref()
            .map(|v| CubridValue::String((**v).to_owned()))
            .unwrap_or(CubridValue::Null),
        SeaValue::Char(v) => v
            .map(|v| CubridValue::String(v.to_string()))
            .unwrap_or(CubridValue::Null),
        SeaValue::Bytes(v) => v
            .as_ref()
            .map(|v| CubridValue::Bytes((**v).to_owned()))
            .unwrap_or(CubridValue::Null),
        _ => CubridValue::String(value.to_string()),
    }
}

/// Convert CUBRID protocol values into SeaQuery values, preserving the requested column name.
pub fn cubrid_value_to_sea(value: &CubridValue, col_name: &str) -> (String, SeaValue) {
    let sea_value = match value {
        CubridValue::Null => SeaValue::String(None),
        CubridValue::Bool(v) => SeaValue::Bool(Some(*v)),
        CubridValue::Short(v) => SeaValue::SmallInt(Some(*v)),
        CubridValue::Int(v) => SeaValue::Int(Some(*v)),
        CubridValue::Long(v) => SeaValue::BigInt(Some(*v)),
        CubridValue::Float(v) => SeaValue::Float(Some(*v)),
        CubridValue::Double(v) => SeaValue::Double(Some(*v)),
        CubridValue::String(v) => SeaValue::String(Some(Box::new(v.clone()))),
        CubridValue::Bytes(v) => SeaValue::Bytes(Some(Box::new(v.clone()))),
        CubridValue::Date { year, month, day } => {
            SeaValue::String(Some(Box::new(format!("{year:04}-{month:02}-{day:02}"))))
        }
        CubridValue::Time {
            hour,
            minute,
            second,
        } => SeaValue::String(Some(Box::new(format!("{hour:02}:{minute:02}:{second:02}")))),
        CubridValue::Timestamp {
            year,
            month,
            day,
            hour,
            minute,
            second,
        } => SeaValue::String(Some(Box::new(format!(
            "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}"
        )))),
        CubridValue::Datetime {
            year,
            month,
            day,
            hour,
            minute,
            second,
            ms,
        } => SeaValue::String(Some(Box::new(format!(
            "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{ms:03}"
        )))),
    };

    (col_name.to_owned(), sea_value)
}
