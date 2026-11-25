pub mod annotation;
pub mod checks;
pub mod discover;
pub mod maps;
pub mod mission;
pub mod report;
pub mod versions;

use std::ops::Range;

use hemtt_config::{Class, Config, Number, Property};

pub trait GetChildren<'a> {
    fn get_children(&'a self) -> &'a [Property];
}

impl<'a, T: GetChildren<'a>> GetChildren<'a> for &'a T {
    fn get_children(&'a self) -> &'a [Property] {
        (*self).get_children()
    }
}

impl<'a> GetChildren<'a> for Property {
    fn get_children(&'a self) -> &'a [Property] {
        match self {
            Property::Class(class) => class.get_children(),
            _ => &[],
        }
    }
}

impl<'a> GetChildren<'a> for Config {
    fn get_children(&'a self) -> &'a [Property] {
        &self.0
    }
}

impl<'a> GetChildren<'a> for Class {
    fn get_children(&'a self) -> &'a [Property] {
        match self {
            Class::Local { properties, .. } => properties,
            _ => &[],
        }
    }
}

pub fn get_class<'a>(parent: &'a dyn GetChildren<'a>, path: &str) -> Option<&'a Class> {
    fn get_child<'a>(props: &'a [Property], name: &str) -> Option<&'a Class> {
        props
            .iter()
            .find(|c| {
                if let Property::Class(class) = c {
                    if class.name().is_some_and(|n| n.as_str() == name) {
                        return true;
                    }
                }
                false
            })
            .map(|c| {
                let Property::Class(class) = c else {
                    panic!("Invalid class after check")
                };
                class
            })
    }
    let mut parts = path.split('.');
    let mut class = get_child(parent.get_children(), parts.next()?)?;
    for part in parts {
        class = get_child(class.get_children(), part)?;
    }
    Some(class)
}

/// The span of a class name, used to point annotations at a class in the file
pub fn class_span(class: &Class) -> Range<usize> {
    class.name().map(|n| n.span.clone()).unwrap_or(0..0)
}

pub fn get_number<'a>(parent: &'a dyn GetChildren<'a>, path: &str) -> Option<(i32, Range<usize>)> {
    for prop in parent.get_children() {
        if let Property::Entry { name, value, .. } = prop {
            if name.as_str() == path {
                if let hemtt_config::Value::Number(hemtt_config::Number::Int32 { value, span }) =
                    value
                {
                    return Some((*value, span.clone()));
                }
            }
        }
    }
    None
}

pub fn get_float<'a>(parent: &'a dyn GetChildren<'a>, path: &str) -> Option<(f32, Range<usize>)> {
    for prop in parent.get_children() {
        if let Property::Entry { name, value, .. } = prop {
            if name.as_str() == path {
                match value {
                    hemtt_config::Value::Number(hemtt_config::Number::Float32 { value, span }) => {
                        return Some((*value, span.clone()));
                    }
                    hemtt_config::Value::Number(hemtt_config::Number::Int32 { value, span }) => {
                        // Convert integer to float
                        return Some((*value as f32, span.clone()));
                    }
                    hemtt_config::Value::Number(hemtt_config::Number::Int64 { value, span }) => {
                        // Convert integer to float
                        return Some((*value as f32, span.clone()));
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

pub fn get_string<'a>(
    parent: &'a dyn GetChildren<'a>,
    path: &str,
) -> Option<(&'a str, &'a Range<usize>)> {
    for prop in parent.get_children() {
        if let Property::Entry { name, value, .. } = prop {
            if name.as_str() == path {
                if let hemtt_config::Value::Str(value) = value {
                    return Some((value.value(), value.span()));
                }
            }
        }
    }
    None
}

pub fn get_array<'a>(
    parent: &'a dyn GetChildren<'a>,
    path: &str,
) -> Option<(&'a [hemtt_config::Item], &'a Range<usize>)> {
    for prop in parent.get_children() {
        if let Property::Entry { name, value, .. } = prop {
            if name.as_str() == path {
                if let hemtt_config::Value::Array(array) = value {
                    return Some((array.items(), array.span()));
                }
            }
        }
    }
    None
}

pub fn extract_number(number: &Number) -> Option<f32> {
    match number {
        Number::Int32 { value, .. } => Some(*value as f32),
        Number::Int64 { value, .. } => Some(*value as f32),
        Number::Float32 { value, .. } => Some(*value),
    }
}
