//! Contains `/Objects` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};


#[derive(Debug, Default, Clone)]
pub struct Objects;

#[derive(Debug)]
pub struct ObjectsLoader<'a> {
    objects: &'a mut Objects,
    definitions: &'a Definitions,
}

impl<'a> ObjectsLoader<'a> {
    pub fn new(objects: &'a mut Objects, definitions: &'a Definitions) -> Self {
        ObjectsLoader {
            objects: objects,
            definitions: definitions,
        }
    }
}

impl<'a> NodeLoaderCommon for ObjectsLoader<'a> {
    type Target = ();

    fn on_finish(self) -> Result<Self::Target> {
        Ok(())
    }
}

impl<'a, R: Read> NodeLoader<R> for ObjectsLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        try!(ignore_current_node(reader));
        Ok(())
    }
}
