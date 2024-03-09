use crate::{
    error::TrackerError,
    field::{AllowedValues, Bound, Field, FieldValue},
};
use actix_web::{body::BoxBody, HttpRequest, HttpResponse, Responder};
use sea_query::Order;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use strum::{AsRefStr, EnumIter, EnumString, IntoEnumIterator};

pub const FIRST_PAGE: u64 = 1;
pub const MAX_PAGE_SIZE: u64 = 500;
pub const DEFAULT_PAGE_SIZE: u64 = 100;

#[derive(Debug, Deserialize, Serialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub metadata: PageMetadata,
}

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct PageMetadata {
    pub total_results: u64,
    pub total_pages: u64,
    pub current_page: u64,
    pub next_page: Option<u64>,
    pub prev_page: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRequestRaw {
    pub page: Option<String>,
    pub size: Option<String>,
    pub sorts: Vec<String>,
}

#[derive(Debug, Copy, Clone, Default, AsRefStr, EnumIter, EnumString)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

#[derive(Debug, Clone, Default)]
pub struct Sort<T: Field> {
    pub field: T,
    pub direction: SortDirection,
}

#[derive(Debug, Clone)]
pub struct PageRequest<T: Field> {
    pub page: u64,
    pub size: u64,
    pub sorts: Vec<Sort<T>>,
}

impl From<SortDirection> for Order {
    fn from(value: SortDirection) -> Self {
        match value {
            SortDirection::Asc => Order::Asc,
            SortDirection::Desc => Order::Desc,
        }
    }
}

impl From<SortDirection> for String {
    fn from(value: SortDirection) -> Self {
        value.as_ref().to_owned()
    }
}

impl<T: Field> TryFrom<PageRequestRaw> for PageRequest<T> {
    type Error = TrackerError;

    fn try_from(page_request: PageRequestRaw) -> Result<Self, Self::Error> {
        let mut sorts: Vec<Sort<T>> = Vec::with_capacity(page_request.sorts.len());
        for sort_raw in page_request.sorts {
            sorts.push(Sort::try_from(sort_raw)?);
        }

        if sorts.is_empty() {
            sorts.push(Sort::<T>::default());
        }

        let page = page_request
            .page
            .map(|page| {
                u64::from_str_radix(&page, 10).map_err(|_| {
                    TrackerError::invalid_field(
                        FieldValue::new("page", page),
                        AllowedValues::integer_min(Bound::inclusive(1)),
                    )
                })
            })
            .transpose()?;

        let size = page_request
            .size
            .map(|size| {
                u64::from_str_radix(&size, 10).map_err(|_| {
                    TrackerError::invalid_field(
                        FieldValue::new("size", size),
                        AllowedValues::integer_min(Bound::inclusive(1)),
                    )
                })
            })
            .transpose()?;

        Ok(Self {
            page: page.unwrap_or(FIRST_PAGE).max(FIRST_PAGE),
            size: size.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE),
            sorts,
        })
    }
}

impl<T: Field> TryFrom<String> for Sort<T> {
    type Error = TrackerError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if let Some((field_raw, dir_raw)) = value.split_once(':') {
            let field = T::from_str(&field_raw).map_err(|_| {
                TrackerError::invalid_field(
                    FieldValue::new("sort:field", field_raw),
                    AllowedValues::choice(T::values()),
                )
            })?;
            let dir = SortDirection::from_str(&dir_raw).map_err(|_| {
                TrackerError::invalid_field(
                    FieldValue::new("sort:direction", dir_raw),
                    AllowedValues::choice(SortDirection::iter()),
                )
            })?;

            Ok(Self {
                field,
                direction: dir,
            })
        } else {
            let field = T::from_str(&value).map_err(|_| {
                TrackerError::invalid_field(
                    FieldValue::new("sort:field", value),
                    AllowedValues::choice(T::values()),
                )
            })?;
            Ok(Self {
                field,
                direction: SortDirection::default(),
            })
        }
    }
}

impl<T: Field> PageRequest<T> {
    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.size
    }
}

impl<T> Page<T> {
    pub fn new(data: Vec<T>, metadata: PageMetadata) -> Self {
        Self { data, metadata }
    }

    pub fn map<U, F>(self, mut f: F) -> Page<U>
    where
        F: FnMut(T) -> U,
    {
        let mut new_data: Vec<U> = Vec::with_capacity(self.data.len());
        for e in self.data {
            new_data.push(f(e));
        }

        Page {
            data: new_data,
            metadata: self.metadata,
        }
    }

    pub fn convert<U>(self) -> Page<U>
    where
        T: Into<U>,
    {
        self.map(|d| d.into())
    }
}

impl PageMetadata {
    pub fn new(page: u64, size: u64, total_results: u64) -> PageMetadata {
        let total_pages = f32::ceil(total_results as f32 / size as f32) as u64;
        PageMetadata {
            total_results,
            total_pages,
            current_page: page,
            next_page: if page < total_pages {
                Some(page + 1)
            } else {
                None
            },
            prev_page: if page > 1 { Some(page - 1) } else { None },
        }
    }
}

impl<T: Serialize> Responder for Page<T> {
    type Body = BoxBody;

    fn respond_to(self, _: &HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::Ok().json(self)
    }
}
