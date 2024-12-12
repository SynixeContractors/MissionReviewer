pub mod annotation;
pub mod checks;
pub mod mission;
pub mod versions;

use std::ops::Range;

use hemtt_config::{Class, Config, Property};

pub trait GetChildren {
    fn get_children(&self) -> Vec<Property>;
}

impl GetChildren for Property {
    fn get_children(&self) -> Vec<Property> {
        match self {
            Property::Class(class) => class.get_children(),
            _ => vec![],
        }
    }
}

impl GetChildren for Config {
    fn get_children(&self) -> Vec<Property> {
        self.0.clone()
    }
}

impl GetChildren for &Config {
    fn get_children(&self) -> Vec<Property> {
        self.0.clone()
    }
}

impl GetChildren for Class {
    fn get_children(&self) -> Vec<Property> {
        match self {
            Class::Local { properties, .. } => properties.clone(),
            _ => vec![],
        }
    }
}

impl GetChildren for &Class {
    fn get_children(&self) -> Vec<Property> {
        match self {
            Class::Local { properties, .. } => properties.clone(),
            _ => vec![],
        }
    }
}

pub fn get_class(parent: impl GetChildren, path: &str) -> Option<Class> {
    fn get_child(props: Vec<Property>, name: &str) -> Option<Class> {
        props
            .iter()
            .find(|c| {
                if let Property::Class(class) = c {
                    if class.name().unwrap().as_str() == name {
                        return true;
                    }
                }
                false
            })
            .map(|c| {
                let Property::Class(class) = c else {
                    panic!("Invalid class after check")
                };
                class.clone()
            })
    }
    let mut ret = None;
    let mut root: Box<dyn GetChildren> = Box::new(parent);
    for part in path.split('.') {
        if let Some(class) = get_child(root.get_children(), part) {
            root = Box::new(class.clone());
            ret = Some(class);
        } else {
            return None;
        }
    }
    ret
}

pub fn get_number(parent: impl GetChildren, path: &str) -> Option<(i32, Range<usize>)> {
    for prop in parent.get_children() {
        if let Property::Entry { name, value, .. } = prop {
            if name.as_str() == path {
                if let hemtt_config::Value::Number(hemtt_config::Number::Int32 { value, span }) =
                    value
                {
                    return Some((value, span));
                }
            }
        }
    }
    None
}

pub fn get_string(parent: impl GetChildren, path: &str) -> Option<(String, Range<usize>)> {
    for prop in parent.get_children() {
        if let Property::Entry { name, value, .. } = prop {
            if name.as_str() == path {
                if let hemtt_config::Value::Str(value) = value {
                    return Some((value.value().to_string(), value.span()));
                }
            }
        }
    }
    None
}
