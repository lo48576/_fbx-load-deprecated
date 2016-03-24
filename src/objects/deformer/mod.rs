//! Contains `/Objects/Deformer` node-related stuff.

pub use self::blend_shape::BlendShape;
pub use self::skin::{Skin, SkinningType};

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::UnknownObject;
use objects::properties::ObjectProperties;
use self::blend_shape::BlendShapeLoader;
use self::skin::SkinLoader;

mod blend_shape;
mod skin;


#[derive(Debug, Clone)]
pub enum Deformer {
    BlendShape(BlendShape),
    Skin(Skin),
    Unknown(UnknownObject),
}

#[derive(Debug)]
pub enum DeformerLoader<'a> {
    BlendShape(BlendShapeLoader<'a>),
    Skin(SkinLoader<'a>),
    Unknown(&'a ObjectProperties<'a>),
}

impl<'a> DeformerLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        match obj_props.subclass {
            "BlendShape" => DeformerLoader::BlendShape(BlendShapeLoader::new(definitions, obj_props)),
            "Skin" => DeformerLoader::Skin(SkinLoader::new(definitions, obj_props)),
            _ => DeformerLoader::Unknown(obj_props),
        }
    }
}

impl<'a> NodeLoaderCommon for DeformerLoader<'a> {
    type Target = Option<Deformer>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(match self {
            DeformerLoader::BlendShape(loader) => try!(loader.on_finish()).map(Deformer::BlendShape),
            DeformerLoader::Skin(loader) => try!(loader.on_finish()).map(Deformer::Skin),
            DeformerLoader::Unknown(obj_props) => Some(Deformer::Unknown(UnknownObject::from_object_properties(obj_props))),
        })
    }
}

impl<'a, R: Read> NodeLoader<R> for DeformerLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        match *self {
            DeformerLoader::BlendShape(ref mut loader) => loader.on_child_node(reader, node_info),
            DeformerLoader::Skin(ref mut loader) => loader.on_child_node(reader, node_info),
            DeformerLoader::Unknown(_) => {
                try!(ignore_current_node(reader));
                Ok(())
            },
        }
    }
}
