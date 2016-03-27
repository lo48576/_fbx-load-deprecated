//! Contains stuff related to `Collection` and `CollectionExclusive` nodes.

pub use self::display_layer::DisplayLayer;

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo};
use objects::properties::ObjectProperties;
use self::display_layer::DisplayLayerLoader;

pub mod display_layer;


#[derive(Debug, Clone)]
pub enum CollectionExclusive {
    DisplayLayer(DisplayLayer),
}

#[derive(Debug)]
pub enum CollectionExclusiveLoader<'a> {
    DisplayLayer(DisplayLayerLoader<'a>),
}

impl<'a> CollectionExclusiveLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Option<Self> {
        match obj_props.subclass {
            "DisplayLayer" => Some(CollectionExclusiveLoader::DisplayLayer(DisplayLayerLoader::new(definitions, obj_props))),
            v => {
                warn!("Unknown subclass ({}) for `/Objects/CollectionExclusive`", v);
                None
            },
        }
    }
}

impl<'a> NodeLoaderCommon for CollectionExclusiveLoader<'a> {
    type Target = Option<CollectionExclusive>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(match self {
            CollectionExclusiveLoader::DisplayLayer(loader) => try!(loader.on_finish()).map(CollectionExclusive::DisplayLayer),
        })
    }
}

impl<'a, R: Read> NodeLoader<R> for CollectionExclusiveLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        match *self {
            CollectionExclusiveLoader::DisplayLayer(ref mut loader) => loader.on_child_node(reader, node_info),
        }
    }
}
