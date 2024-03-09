use crate::data::SortDirection;
use chrono::{DateTime, Utc};
use sea_query::ColumnRef;
use serde::{Deserialize, Serialize};
use std::{fmt, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

const ISO_FORMAT: &str = "yyyy-mm-ddTHH:MM:ss[.SSS]Z";

#[derive(Debug, Error)]
#[error("`{0}` is invalid.")]
pub struct InvalidFieldError(pub String);

pub trait Field: ToString + FromStr + Copy + Default {
    fn column(&self) -> ColumnRef;

    fn name(&self) -> String;

    fn values() -> impl Iterator<Item = Self>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Uuid(Uuid),
    String(String),
    Int8(i8),
    Int16(i16),
    Int32(i32),
    Int64(i64),
    Uint8(u8),
    Uint16(u16),
    Uint32(u32),
    Uint64(u64),
    Float(f64),
    DateTime(DateTime<Utc>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValue {
    pub name: String,
    pub value: Option<Value>,
}

#[derive(Debug, Clone)]
pub struct FieldValues(pub Vec<FieldValue>);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bound {
    pub value: Value,
    pub inclusive: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum AllowedValues {
    Choice {
        values: Vec<Value>,
    },
    Integer {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<Bound>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<Bound>,
    },
    Float {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<Bound>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<Bound>,
    },
    DateTime {
        format: String,
    },
    String {
        #[serde(skip_serializing_if = "Option::is_none")]
        min_length: Option<usize>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max_length: Option<usize>,
    },
}

impl From<Uuid> for Value {
    fn from(value: Uuid) -> Self {
        Self::Uuid(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&String> for Value {
    fn from(value: &String) -> Self {
        Self::String(value.clone())
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.to_owned())
    }
}

impl From<SortDirection> for Value {
    fn from(value: SortDirection) -> Self {
        Self::String(value.as_ref().to_owned())
    }
}

impl<T: Field + Copy> From<T> for Value {
    fn from(value: T) -> Self {
        Self::String(value.name())
    }
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Self::Int64(value)
    }
}

impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Self::Int32(value)
    }
}

impl From<i16> for Value {
    fn from(value: i16) -> Self {
        Self::Int16(value)
    }
}

impl From<i8> for Value {
    fn from(value: i8) -> Self {
        Self::Int8(value)
    }
}

impl From<u64> for Value {
    fn from(value: u64) -> Self {
        Self::Uint64(value)
    }
}

impl From<u32> for Value {
    fn from(value: u32) -> Self {
        Self::Uint32(value)
    }
}

impl From<u16> for Value {
    fn from(value: u16) -> Self {
        Self::Uint16(value)
    }
}

impl From<u8> for Value {
    fn from(value: u8) -> Self {
        Self::Uint8(value)
    }
}

impl From<f32> for Value {
    fn from(value: f32) -> Self {
        Self::Float(value as f64)
    }
}

impl From<f64> for Value {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl From<DateTime<Utc>> for Value {
    fn from(value: DateTime<Utc>) -> Self {
        Self::DateTime(value)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Uuid(v) => write!(f, "{0}", v),
            Self::String(v) => write!(f, "{0}", v),
            Self::Int64(v) => write!(f, "{0}", v),
            Self::Int32(v) => write!(f, "{0}", v),
            Self::Int16(v) => write!(f, "{0}", v),
            Self::Int8(v) => write!(f, "{0}", v),
            Self::Uint64(v) => write!(f, "{0}", v),
            Self::Uint32(v) => write!(f, "{0}", v),
            Self::Uint16(v) => write!(f, "{0}", v),
            Self::Uint8(v) => write!(f, "{0}", v),
            Self::Float(v) => write!(f, "{0}", v),
            Self::DateTime(v) => write!(f, "{0}", v),
        }
    }
}

impl FieldValue {
    pub fn new<F: Into<String>, V: Into<Value>>(name: F, value: V) -> FieldValue {
        FieldValue {
            name: name.into(),
            value: Some(value.into()),
        }
    }

    pub fn null_value<F: Into<String>>(name: F) -> FieldValue {
        FieldValue {
            name: name.into(),
            value: None,
        }
    }
}

impl Bound {
    pub fn inclusive<T: Into<Value>>(value: T) -> Bound {
        Bound {
            value: value.into(),
            inclusive: true,
        }
    }

    pub fn exclusive<T: Into<Value>>(value: T) -> Bound {
        Bound {
            value: value.into(),
            inclusive: false,
        }
    }
}

impl fmt::Display for Bound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.inclusive {
            write!(f, ">= {0}", self.value)
        } else {
            write!(f, "> {0}", self.value)
        }
    }
}

impl AllowedValues {
    pub fn choice<T: Into<Value>, I: IntoIterator<Item = T>>(values: I) -> AllowedValues {
        AllowedValues::Choice {
            values: values.into_iter().map(|v| v.into()).collect(),
        }
    }

    pub fn integer_between(min: Bound, max: Bound) -> AllowedValues {
        AllowedValues::Integer {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn integer_min(min: Bound) -> AllowedValues {
        AllowedValues::Integer {
            min: Some(min),
            max: None,
        }
    }

    pub fn integer_max(max: Bound) -> AllowedValues {
        AllowedValues::Integer {
            min: None,
            max: Some(max),
        }
    }

    pub fn float_between(min: Bound, max: Bound) -> AllowedValues {
        AllowedValues::Float {
            min: Some(min),
            max: Some(max),
        }
    }

    pub fn float_min(min: Bound) -> AllowedValues {
        AllowedValues::Float {
            min: Some(min),
            max: None,
        }
    }

    pub fn float_max(max: Bound) -> AllowedValues {
        AllowedValues::Float {
            min: None,
            max: Some(max),
        }
    }

    pub fn datetime_iso() -> AllowedValues {
        AllowedValues::DateTime {
            format: ISO_FORMAT.to_owned(),
        }
    }

    pub fn datetime(format: String) -> AllowedValues {
        AllowedValues::DateTime { format }
    }

    pub fn string_len_between(min_length: usize, max_length: usize) -> AllowedValues {
        AllowedValues::String {
            min_length: Some(min_length),
            max_length: Some(max_length),
        }
    }

    pub fn string_len_min(min_length: usize) -> AllowedValues {
        AllowedValues::String {
            min_length: Some(min_length),
            max_length: None,
        }
    }

    pub fn string_len_max(max_length: usize) -> AllowedValues {
        AllowedValues::String {
            min_length: None,
            max_length: Some(max_length),
        }
    }
}

impl fmt::Display for AllowedValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Choice { values } => {
                write!(
                    f,
                    "Allowed values are: {0}",
                    values
                        .iter()
                        .map(|e| e.to_string())
                        .collect::<Vec<String>>()
                        .join(", ")
                )
            }
            Self::Integer { min, max } => match (min, max) {
                (Some(min), Some(max)) => {
                    write!(f, "Value must be an integer {0} and {1}.", min, max)
                }
                (Some(min), None) => {
                    write!(f, "Value must be  an integer {0}.", min)
                }
                (None, Some(max)) => {
                    write!(f, "Value must be an integer {0}.", max)
                }
                (None, None) => write!(f, "Value must be an integer."),
            },
            Self::Float { min, max } => match (min, max) {
                (Some(min), Some(max)) => {
                    write!(f, "Value must be a number {0} and {1}.", min, max)
                }
                (Some(min), None) => {
                    write!(f, "Value must be a number {0}.", min)
                }
                (None, Some(max)) => {
                    write!(f, "Value must be a number {0}.", max)
                }
                (None, None) => write!(f, "Value must be a number."),
            },
            Self::DateTime { format } => {
                write!(f, "Must be a date time in the format `{0}`.", format)
            }
            Self::String {
                min_length,
                max_length,
            } => match (min_length, max_length) {
                (Some(min), Some(max)) => write!(
                    f,
                    "Value must be between {} and {} characters long.",
                    min, max
                ),
                (None, Some(max)) => write!(f, "Value must be at most {} characters long.", max),
                (Some(min), None) => write!(f, "Value must be at least {} characters long.", min),
                (None, None) => panic!("Improperly constructed String"),
            },
        }
    }
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{0} `{1}`", self.name, format_value(&self.value))
    }
}

impl From<FieldValue> for FieldValues {
    fn from(value: FieldValue) -> Self {
        Self(vec![value])
    }
}

impl<I: IntoIterator<Item = FieldValue>> From<I> for FieldValues {
    fn from(value: I) -> Self {
        Self(Vec::from_iter(value))
    }
}

impl fmt::Display for FieldValues {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{0}",
            self.0
                .iter()
                .map(|e| e.to_string())
                .collect::<Vec<String>>()
                .join(", ")
        )
    }
}

pub fn format_value(value: &Option<Value>) -> String {
    value
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or("null".to_owned())
}

#[macro_export]
macro_rules! field_names {
    (
        $type_name: ident<$column_type: ty> {
            $(
                $(#[$( $default:tt )+])?
                $variant_name:ident $( ( $sub_field_type:ty ) )? => { $($variant_args:tt)+ }
            ),+
            $(,)?
        }
    ) => {
        use sea_query::{ColumnRef, IntoColumnRef};

        #[derive(Debug, Copy, Clone)]
        pub enum $type_name {
            $(
                $variant_name $( ( $sub_field_type ) )?
            ),+
        }

        impl $crate::field::Field for $type_name {
            fn column(&self) -> ColumnRef {
                match self {
                    $(
                        field_names!(@variant_match_arm(field) {
                            $variant_name$( ( $sub_field_type ) )?
                        })
                        =>
                        field_names!(@column(field, $column_type) {
                            $variant_name$( ( $sub_field_type ) )? => $($variant_args)+
                        })
                    ),+
                }
            }

            fn name(&self) -> String {
                match self {
                    $(
                        field_names!(@variant_match_arm(field) {
                            $variant_name$( ( $sub_field_type ) )?
                        })
                        =>
                        field_names!(@name(field) {
                            $variant_name$( ( $sub_field_type ) )? => $($variant_args)+
                        })
                    ),+
                }
            }

            fn values() -> impl Iterator<Item = Self> {
                static VALUES: once_cell::sync::Lazy<Vec<$type_name>> = once_cell::sync::Lazy::new(|| {
                    let capacity = field_names!(@values_size [$( ( $( $sub_field_type )? ) ),+]);
                    let mut values = Vec::with_capacity(capacity);
                    $(
                        field_names!(@append_values(values, $type_name) $variant_name($( $sub_field_type )?));
                    )+

                    values
                });

                VALUES.iter().copied()
            }
        }

        impl Default for $type_name {
            fn default() -> Self {
                field_names!(@default [
                    $(
                        $(#[$($default)+])? $variant_name $( ( $sub_field_type ) )?
                    ),+
                ])
            }
        }

        impl std::fmt::Display for $type_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{0}", self.name())
            }
        }

        impl std::str::FromStr for $type_name {
            type Err = $crate::field::InvalidFieldError;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                field_names!(@from_str(s) {
                    $(
                        ( $variant_name $( ( $sub_field_type ) )? => { $($variant_args)+ } )
                    ),+
                })
            }
        }

        impl From<$type_name> for String {
            fn from(value: $type_name) -> String {
                value.name()
            }
        }
    };

    (
        @variant_match_arm($field:pat_param) {
            $variant_name:ident
        }
    ) => {
        Self::$variant_name
    };

    (
        @variant_match_arm($field:pat_param) {
            $variant_name:ident($sub_field_type:ty)
        }
    ) => {
        Self::$variant_name ($field)
    };

    (
        @name($field:ident) {
            $variant_name:ident => value: $value:literal $($rest:tt)*
        }
    ) => {
        $value.to_owned()
    };

    (
        @name($field:ident) {
            $variant_name:ident($sub_field_type:ty) => prefix: $prefix:literal
        }
    ) => {
        format!("{0}.{1}", $prefix, $field.name())
    };

    (
        @column($field:ident, $column_type:ty) {
            $variant_name:ident => value: $value:literal
        }
    ) => {
        (<$column_type>::Table, <$column_type>::$variant_name).into_column_ref()
    };

    (
        @column($field:ident, $column_type:ty) {
            $variant_name:ident => value: $value:literal, column: $column:ident
        }
    ) => {
        (<$column_type>::Table, <$column_type>::$column).into_column_ref()
    };

    (
        @column($field:ident, $column_type:ty) {
            $variant_name:ident($sub_field_type:ty) => $($rest:tt)+
        }
    ) => {
        $field.column()
    };

    (
        @values_size [()]
    ) => {
        1
    };

    (
        @values_size [($sub_field_type:ty) ]
    ) => {
        <sub_field_type>::values().count()
    };

    (
        @values_size [(), $( ( $( $rest_sub_field_type:ty )? ) ),+]
    ) => {
        1 + field_names!(@values_size [$( ( $( $rest_sub_field_type )? ) ) ,*])
    };

    (
        @values_size [($sub_field_type:ty), $( ( $( $rest_sub_field_type:ty )? ) ),+]
    ) => {
        <$sub_field_type>::values().count() + field_names!(@values_size [$( ( $( $rest_sub_field_type )? ) ) ,*])
    };

    (
        @append_values($vec:ident, $type_name:ident) $name:ident ()
    ) => {
        $vec.push($type_name::$name)
    };
    (
        @append_values($vec:ident, $type_name:ident) $name:ident ($sub_field_type:ty)
    ) => {
        $vec.extend(<$sub_field_type>::values().map(|v| $type_name::$name(v)))
    };

    (
        @from_str($arg:ident) {
            ( $($first:tt)+ )
            $(, ( $($rest:tt)+ ) )*
        }
    ) => {
        {
            let lower = $arg.to_ascii_lowercase();

            if field_names!(@from_str_cond(lower) $( $first )+ ) {
                field_names!(@from_str_result(lower) $( $first )+ )
            } $( else if field_names!(@from_str_cond(lower) $( $rest )+ ) {
                field_names!(@from_str_result(lower) $( $rest )+ )
            })* else {
                Err($crate::field::InvalidFieldError($arg.to_owned()))
            }
        }
    };

    (
        @from_str_cond($arg:ident) $name:ident => { value: $value:literal $( $rest:tt)* }
    ) => {
        $arg.eq($value)
    };

    (
        @from_str_cond($arg:ident) $name:ident ($sub_field_type:ty) => { prefix: $prefix:literal $($rest:tt)* }
    ) => {
        $arg.starts_with($prefix)
    };

    (
        @from_str_result($arg:ident) $name:ident => { $( $rest:tt)* }
    ) => {
        Ok(Self::$name)
    };

    (
        @from_str_result($arg:ident) $name:ident($sub_field_type:ty) => { prefix: $prefix:literal $($rest:tt)* }
    ) => {
        {
            let stripped = &$arg[$prefix.len()+1..];
            Ok(Self::$name(if stripped.is_empty() {
                <$sub_field_type>::default()
            } else {
                <$sub_field_type>::from_str(stripped)?
            }))
        }
    };

    (
        @default [ #[default] $name:ident $($rest:tt)* ]
    ) => {
        Self::$name
    };

    (
        @default [ #[default($arg_expr: expr)] $name:ident $($rest:tt)* ]
    ) => {
        compiler_error!("Enum variant `{}` does not take an argument.", $name)
    };

    (
        @default [ #[default] $name:ident($sub_field_type:ty) $($rest:tt)* ]
    ) => {
        Self::$name(<$sub_field_type>::default())
    };

    (
        @default [ #[default($arg_expr:expr)] $name:ident($sub_field_type:ty) $($rest:tt)* ]
    ) => {
        Self::$name($arg_expr)
    };

    (
        @default [
            $(#[$( $head_default:tt )+])?
            $head_name:ident $( ( $head_sub_field_type:ty ) )?
            $(,)?
            $(
                $(#[$( $rest_default:tt )+])?
                $rest_head:ident$( ( $rest_sub_field_type:ty ) )?
            ),*
        ]
    ) => {
        field_names!(@default [
            $(
                $(#[$( $rest_default )+])?
                $rest_head $( ( $rest_sub_field_type ) )?
            ),*
        ])
    };

    (
        @default []
    ) => {
        compile_error!("No enum variant tagged with #[default]")
    };
}
