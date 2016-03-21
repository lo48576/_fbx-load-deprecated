//! Contains `/Definitions/PropertyTemplate` node-related stuff.

use std::collections::HashMap;
use std::io::Read;
use fbx_binary_reader::EventReader;
use error::Result;
use node_loader::{NodeLoader, RawNodeInfo, ignore_current_node};
use property::{GenericProperties, GenericPropertiesLoader};


#[derive(Debug, Clone)]
pub struct PropertyTemplate {
    properties: GenericProperties,
}

#[derive(Debug, Default)]
struct PropertyTemplateLoader {
    properties: Option<GenericProperties>,
}

impl PropertyTemplateLoader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<R: Read> NodeLoader<R> for PropertyTemplateLoader {
    type Target = PropertyTemplate;

    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, .. } = node_info;
        match name.as_ref() {
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            _ => {
                error!("Unknown node: `/Definitions/ObjectType/PropertyTemplate/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }

    fn on_finish(self) -> Result<Self::Target> {
        Ok(PropertyTemplate {
            properties: self.properties.unwrap_or_default(),
        })
    }
}

/// Corresponds to `/Definitions/ObjectType` node.
#[derive(Debug, Default, Clone)]
pub struct PropertyTemplates {
    pub templates: HashMap<(String, String), PropertyTemplate>,
}

#[derive(Debug)]
pub struct PropertyTemplatesLoader<'a> {
    templates: &'a mut PropertyTemplates,
    object_type: &'a str,
}

impl<'a> PropertyTemplatesLoader<'a> {
    pub fn new(templates: &'a mut PropertyTemplates, object_type: &'a str) -> Self {
        PropertyTemplatesLoader {
            templates: templates,
            object_type: object_type,
        }
    }
}

impl<'a, R: Read> NodeLoader<R> for PropertyTemplatesLoader<'a> {
    type Target = ();

    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Count" => {
                try!(ignore_current_node(reader));
            },
            "PropertyTemplate" => {
                if let Some(template_name) = properties.iter().next().and_then(|p| p.get_string()) {
                    let template = try!(PropertyTemplateLoader::new().load(reader));
                    self.templates.templates.insert((self.object_type.to_owned(), template_name.to_owned()), template);
                    warn!("Template: ({}, {})", self.object_type, template_name);
                } else {
                    error!("Invalid property at `/Definitions/ObjectType/PropertyTemplate`: type error");
                    try!(ignore_current_node(reader));
                }
            },
            _ => {
                error!("Unknown node: `/Definitions/ObjectType/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }

    fn on_finish(self) -> Result<Self::Target> {
        Ok(())
    }
}
