//! Contains `/Objects/Deformer` node-related stuff.

pub use self::blend_shape::BlendShape;
pub use self::blend_shape_channel::BlendShapeChannel;
pub use self::cluster::Cluster;
pub use self::skin::{Skin, SkinningType};

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo};
use objects::properties::ObjectProperties;
use self::blend_shape::BlendShapeLoader;
use self::blend_shape_channel::BlendShapeChannelLoader;
use self::cluster::ClusterLoader;
use self::skin::SkinLoader;

mod blend_shape;
mod blend_shape_channel;
mod cluster;
mod skin;


#[derive(Debug, Clone)]
pub enum Deformer {
    BlendShape(BlendShape),
    BlendShapeChannel(BlendShapeChannel),
    Cluster(Cluster),
    Skin(Skin),
}

#[derive(Debug)]
pub enum DeformerLoader<'a> {
    BlendShape(BlendShapeLoader<'a>),
    BlendShapeChannel(BlendShapeChannelLoader<'a>),
    Cluster(ClusterLoader<'a>),
    Skin(SkinLoader<'a>),
}

impl<'a> DeformerLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Option<Self> {
        match  obj_props.class {
            "Deformer" => match obj_props.subclass {
                "BlendShape" => Some(DeformerLoader::BlendShape(BlendShapeLoader::new(definitions, obj_props))),
                "Skin" => Some(DeformerLoader::Skin(SkinLoader::new(definitions, obj_props))),
                v => {
                    warn!("Unknown subclass ({}) for `/Objects/Deformer(class={})", v, obj_props.class);
                    None
                },
            },
            "SubDeformer" => match obj_props.subclass {
                "Cluster" => Some(DeformerLoader::Cluster(ClusterLoader::new(definitions, obj_props))),
                "BlendShapeChannel" => Some(DeformerLoader::BlendShapeChannel(BlendShapeChannelLoader::new(definitions, obj_props))),
                v => {
                    warn!("Unknown subclass ({}) for `/Objects/Deformer(class={})", v, obj_props.class);
                    None
                },
            },
            v => {
                warn!("Unknown class ({}) for `/Objects/Deformer`", v);
                None
            },
        }
    }
}

impl<'a> NodeLoaderCommon for DeformerLoader<'a> {
    type Target = Option<Deformer>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(match self {
            DeformerLoader::BlendShape(loader) => try!(loader.on_finish()).map(Deformer::BlendShape),
            DeformerLoader::BlendShapeChannel(loader) => try!(loader.on_finish()).map(Deformer::BlendShapeChannel),
            DeformerLoader::Cluster(loader) => try!(loader.on_finish()).map(Deformer::Cluster),
            DeformerLoader::Skin(loader) => try!(loader.on_finish()).map(Deformer::Skin),
        })
    }
}

impl<'a, R: Read> NodeLoader<R> for DeformerLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        match *self {
            DeformerLoader::BlendShape(ref mut loader) => loader.on_child_node(reader, node_info),
            DeformerLoader::BlendShapeChannel(ref mut loader) => loader.on_child_node(reader, node_info),
            DeformerLoader::Cluster(ref mut loader) => loader.on_child_node(reader, node_info),
            DeformerLoader::Skin(ref mut loader) => loader.on_child_node(reader, node_info),
        }
    }
}
