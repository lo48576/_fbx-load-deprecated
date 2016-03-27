use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader, OptionalProperties};
use super::NodeAttributeType;

#[derive(Debug, Clone, Copy)]
pub struct NullNodeAttribute {
    pub id: i64,
    pub color: [f32; 3],
    pub size: f64,
    pub look: NullNodeLook,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NullNodeLook {
    None,
    Cross,
}

impl NullNodeLook {
    pub fn from_i64(val: i64) -> Option<Self> {
        match val {
            0 => Some(NullNodeLook::None),
            1 => Some(NullNodeLook::Cross),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct NullNodeAttributeLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    properties: Option<GenericProperties>,
    type_flags: Option<NodeAttributeType>,
}

impl<'a> NullNodeAttributeLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        NullNodeAttributeLoader {
            definitions: definitions,
            obj_props: obj_props,
            properties: None,
            type_flags: None,
        }
    }
}

impl<'a> NodeLoaderCommon for NullNodeAttributeLoader<'a> {
    type Target = Option<NullNodeAttribute>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let defaults = self.definitions.templates.templates.get(&("NodeAttribute".to_owned(), "FbxNull".to_owned())).map(|t| &t.properties);
        let color = self.properties.get_or_default(defaults, "Color").and_then(|p| p.value.get_vec_f32().into_iter().find(|v| v.len() >= 3).map(|v| [v[0], v[1], v[2]]));
        let size = self.properties.get_or_default(defaults, "Size").and_then(|p| p.value.get_f64());
        let look = self.properties.get_or_default(defaults, "Look").and_then(|p| p.value.get_i64().and_then(NullNodeLook::from_i64));
        if_all_some!{(
            color=color,
            size=size,
            look=look,
        ) {
            Ok(Some(NullNodeAttribute {
                id: self.obj_props.id,
                color: color,
                size: size,
                look: look,
            }))
        } else {
            error!("Required property not found for `/Objects/NodeAttribute(Null)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for NullNodeAttributeLoader<'a> {
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
                warn!("Unknown node: `/Objects/NodeAttribute(Null)/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
