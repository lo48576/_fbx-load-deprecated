use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader, OptionalProperties};
use super::NodeAttributeType;

#[derive(Debug, Clone, Copy)]
pub struct LimbNodeAttribute {
    pub id: i64,
    pub type_flags: NodeAttributeType,
    pub size: f64,
}

#[derive(Debug)]
pub struct LimbNodeAttributeLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    properties: Option<GenericProperties>,
    type_flags: Option<NodeAttributeType>,
}

impl<'a> LimbNodeAttributeLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        LimbNodeAttributeLoader {
            definitions: definitions,
            obj_props: obj_props,
            properties: None,
            type_flags: None,
        }
    }
}

impl<'a> NodeLoaderCommon for LimbNodeAttributeLoader<'a> {
    type Target = Option<LimbNodeAttribute>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let defaults = self.definitions.templates.templates.get(&("NodeAttribute".to_owned(), "FbxSkeleton".to_owned())).map(|t| &t.properties);
        let size = self.properties.get_or_default(defaults, "Size").and_then(|p| p.value.get_f64());
        if_all_some!{(
            type_flags=self.type_flags,
            size=size,
        ) {
            Ok(Some(LimbNodeAttribute {
                id: self.obj_props.id,
                type_flags: type_flags,
                size: size,
            }))
        } else {
            error!("Required property not found for `/Objects/NodeAttribute(LimbNode)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for LimbNodeAttributeLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            "TypeFlags" => {
                self.type_flags = properties.iter().next().and_then(|p| p.get_string()).map(NodeAttributeType::from_string);
                try!(ignore_current_node(reader));
            },
            _ => {
                warn!("Unknown node: `/Objects/NodeAttribute(LimbNode)/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
