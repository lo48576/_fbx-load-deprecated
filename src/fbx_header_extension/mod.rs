//! Contains `/FBXHeaderExtension` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};


#[derive(Debug, Clone)]
pub struct FbxHeaderExtension;

#[derive(Debug, Default, Clone)]
pub struct FbxHeaderExtensionLoader;

impl FbxHeaderExtensionLoader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl NodeLoaderCommon for FbxHeaderExtensionLoader {
    type Target = FbxHeaderExtension;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(FbxHeaderExtension)
    }
}

impl<R: Read> NodeLoader<R> for FbxHeaderExtensionLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        warn!("Ignoring node: {:?}", node_info);
        try!(ignore_current_node(reader));
        Ok(())
    }
}
