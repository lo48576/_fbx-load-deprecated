//! Contains `Properties70` and `Properties70/P` related stuff.

pub use self::property_node::{PropertyNode, PropertyNodeLoader};
pub use self::property_node_value::PropertyNodeValue;
pub use self::flags::PropertyFlags;

use std::collections::BTreeMap;
use std::io::Read;
use fbx_binary_reader::EventReader;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};

pub mod flags;
pub mod property_node;
pub mod property_node_value;


#[derive(Debug, Default, Clone)]
pub struct GenericProperties {
    pub properties: BTreeMap<String, PropertyNode>,
}

#[derive(Debug)]
pub struct GenericPropertiesLoader {
    properties: BTreeMap<String, PropertyNode>,
}

impl GenericPropertiesLoader {
    pub fn new(_node_version: i32) -> Self {
        GenericPropertiesLoader {
            properties: Default::default(),
        }
    }
}

impl NodeLoaderCommon for GenericPropertiesLoader {
    type Target = GenericProperties;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(GenericProperties {
            properties: self.properties,
        })
    }
}

impl<R: Read> NodeLoader<R> for GenericPropertiesLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "P" => {
                let mut prop_iter = properties.iter();
                if let Some(prop_name) = prop_iter.next().and_then(|p| p.get_string()) {
                    if let Some(loader) = PropertyNodeLoader::new(prop_iter) {
                        if let Some(prop_node) = try!(loader.load(reader)) {
                            self.properties.insert(prop_name.to_owned(), prop_node);
                        }
                    } else {
                        try!(ignore_current_node(reader));
                    }
                } else {
                    error!("Cannot get property name");
                    try!(ignore_current_node(reader));
                }
            },
            _ => {
                error!("Unknown property node: `{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
