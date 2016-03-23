///! Contains FBX Scene related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::{Definitions, DefinitionsLoader};
use error::{Error, Result};
use fbx_header_extension::{FbxHeaderExtension, FbxHeaderExtensionLoader};
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};


#[derive(Debug, Clone)]
pub struct FbxScene {
    pub fbx_header_extension: FbxHeaderExtension,
}

#[derive(Debug, Default)]
pub struct FbxSceneLoader {
    fbx_header_extension: Option<FbxHeaderExtension>,
    definitions: Option<Definitions>,
}

impl FbxSceneLoader {
    pub fn new(_fbx_version: i32) -> Self {
        Default::default()
    }
}

impl NodeLoaderCommon for FbxSceneLoader {
    type Target = FbxScene;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(FbxScene {
            fbx_header_extension: try!(self.fbx_header_extension.ok_or(Error::UnclassifiedCritical("Required node not found".to_owned()))),
        })
    }
}

impl<R: Read> NodeLoader<R> for FbxSceneLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, .. } = node_info;
        match name.as_ref() {
            "FBXHeaderExtension" => {
                self.fbx_header_extension = Some(try!(FbxHeaderExtensionLoader::new().load(reader)));
            },
            "Definitions" => {
                self.definitions = Some(try!(DefinitionsLoader::new().load(reader)));
            },
            _ => {
                warn!("Unknown node: `{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}

pub fn load_scene<R: Read>(reader: &mut EventReader<R>, fbx_version: i32) -> Result<FbxScene> {
    FbxSceneLoader::new(fbx_version).load(reader)
}
