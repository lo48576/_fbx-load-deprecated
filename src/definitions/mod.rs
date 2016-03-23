//! Contains `/Definitions` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use self::template::{PropertyTemplates, PropertyTemplatesLoader};

pub mod template;


#[derive(Debug, Clone)]
pub struct Definitions {
    pub templates: PropertyTemplates,
}

#[derive(Debug, Default)]
pub struct DefinitionsLoader {
    pub templates: PropertyTemplates,
}

impl DefinitionsLoader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl NodeLoaderCommon for DefinitionsLoader {
    type Target = Definitions;

    fn on_finish(self) -> Result<Self::Target> {
debug!("Definitions.templates: {:#?}", self.templates);
        Ok(Definitions {
            templates: self.templates,
        })
    }
}

impl<R: Read> NodeLoader<R> for DefinitionsLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Definitions` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Definitions/Version`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "Count" => {
                try!(ignore_current_node(reader));
            },
            "ObjectType" => {
                if let Some(object_type) = properties.iter().next().and_then(|p| p.get_string()) {
                    try!(PropertyTemplatesLoader::new(&mut self.templates, object_type).load(reader));
                } else {
                    error!("Invalid property at `/Definitions/ObjectType`: type error");
                    try!(ignore_current_node(reader));
                }
            },
            _ => {
                error!("Unknown node: `/Definitions/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
