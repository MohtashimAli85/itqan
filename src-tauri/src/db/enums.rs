use rusqlite::Row;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::AppError;

pub fn to_text<T: Serialize>(value: &T) -> Result<String, AppError> {
    serde_json::to_value(value)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| AppError::InvalidInput("expected a text value".into()))
}

fn from_text<T: DeserializeOwned>(index: usize, value: String) -> rusqlite::Result<T> {
    serde_json::from_value(serde_json::Value::String(value)).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, rusqlite::types::Type::Text, error.into())
    })
}

pub fn column<T: DeserializeOwned>(row: &Row, index: usize) -> rusqlite::Result<T> {
    from_text(index, row.get(index)?)
}

pub fn optional_column<T: DeserializeOwned>(
    row: &Row,
    index: usize,
) -> rusqlite::Result<Option<T>> {
    row.get::<_, Option<String>>(index)?
        .map(|value| from_text(index, value))
        .transpose()
}
