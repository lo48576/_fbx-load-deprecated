//! Contains `/Objects/Deformer(BlendShape)` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;


#[derive(Debug, Clone)]
pub struct BlendShape {
    pub id: i64,
    pub name: String,
}

#[derive(Debug)]
pub struct BlendShapeLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
}

impl<'a> BlendShapeLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        BlendShapeLoader {
            definitions: definitions,
            obj_props: obj_props,
        }
    }
}

impl<'a> NodeLoaderCommon for BlendShapeLoader<'a> {
    type Target = Option<BlendShape>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(Some(BlendShape {
            id: self.obj_props.id,
            name: self.obj_props.name.to_owned(),
        }))
    }
}

impl<'a, R: Read> NodeLoader<R> for BlendShapeLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Deformer(BlendShape)` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Deformer(BlendShape)/Version`: type error");
                    },
                }
            },
            _ => {
                warn!("Unknown node: `/Objects/Deformer(BlendShape)/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
