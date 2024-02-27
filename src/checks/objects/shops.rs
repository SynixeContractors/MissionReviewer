use std::path::Path;

use hemtt_common::reporting::Processed;
use hemtt_config::Config;

use crate::{
    annotation::{Annotation, Level},
    get_class, get_number, get_string, GetChildren,
};

use super::ObjectCheck;

const REQUIRED_SHOPS: usize = 2;

pub struct ShopCheck {
    messages: Vec<Annotation>,
    count: usize,
}

impl ShopCheck {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            count: 0,
        }
    }

    fn check_attribute(
        &mut self,
        dir: &Path,
        attributes: impl GetChildren,
        property: &str,
        message: String,
    ) {
        if let Some((value, span)) = get_number(attributes, property) {
            if value == 1 {
                self.messages.push(Annotation::new(
                    None,
                    dir.join("mission.sqm").display().to_string(),
                    span,
                    message,
                    Level::Error,
                ));
            }
        }
    }
}

impl Default for ShopCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl ObjectCheck for ShopCheck {
    fn observe(
        &mut self,
        _: (&Processed, &Config),
        dir: &Path,
        class: &hemtt_config::Class,
        data_type: &str,
    ) {
        if data_type != "Object" {
            return;
        }
        let Some(custom_attributes) = get_class(class, "CustomAttributes") else {
            return;
        };
        for attribute in custom_attributes.get_children() {
            let Some((property, _)) = get_string(attribute, "property") else {
                continue;
            };
            if property == "crate_client_gear_attribute_shop" {
                if let Some(attributes) = get_class(class, "Attributes") {
                    self.check_attribute(
                        dir,
                        &attributes,
                        "createAsSimpleObject",
                        "shops must not be simple objects".to_string(),
                    );
                    self.check_attribute(
                        dir,
                        &attributes,
                        "createAsSimpleObject",
                        "shops must not be simple objects".to_string(),
                    );
                    // self.check_attribute(
                    //     dir,
                    //     &attributes,
                    //     "simulation",
                    //     "shops must not have simulation disabled".to_string(),
                    // );
                }
                self.count += 1;
            }
        }
    }

    fn done(&self, dir: &Path) -> Vec<Annotation> {
        let mut messages = self.messages.clone();
        if self.count == 0 {
            messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                "No shops found".to_string(),
                Level::Error,
            ));
        } else if self.count < REQUIRED_SHOPS {
            messages.push(Annotation::new(
                None,
                dir.join("mission.sqm").display().to_string(),
                0..0,
                format!(
                    "Not enough shops found, at least {} are required",
                    REQUIRED_SHOPS
                ),
                Level::Error,
            ));
        }
        messages
    }
}
