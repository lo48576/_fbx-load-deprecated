///! Contains FBX Scene related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::{Definitions, DefinitionsLoader};
use error::{Error, Result};
use fbx_header_extension::{FbxHeaderExtension, FbxHeaderExtensionLoader};
use node_loader::{FormatConvert, NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use connections::{Connection, ConnectionsLoader};
use objects::{Objects, ObjectsLoader};


#[derive(Debug)]
pub struct FbxScene<I> {
    pub fbx_header_extension: FbxHeaderExtension,
    pub objects: Objects<I>,
    pub connections: Vec<Connection>,
}

impl<I> FbxScene<I> {
    pub fn triangulate<F>(&mut self, triangulator: F)
        where F: Fn(&[[f32; 3]], &[u32], &mut Vec<u32>) -> u32
    {
        for (_id, mesh) in &mut self.objects.geometry_meshes {
            mesh.triangulate(&triangulator);
        }
    }
}

impl<I: Clone> Clone for FbxScene<I> {
    fn clone(&self) -> Self {
        FbxScene {
            fbx_header_extension: self.fbx_header_extension.clone(),
            objects: self.objects.clone(),
            connections: self.connections.clone(),
        }
    }
}

#[derive(Debug)]
pub struct FbxSceneLoader<C: FormatConvert> {
    converter: C,
    fbx_header_extension: Option<FbxHeaderExtension>,
    definitions: Option<Definitions>,
    objects: Objects<C::ImageResult>,
    connections: Option<Vec<Connection>>,
}

impl<C: FormatConvert>  FbxSceneLoader<C> {
    pub fn new(_fbx_version: i32, converter: C) -> Self {
        FbxSceneLoader {
            converter: converter,
            fbx_header_extension: None,
            definitions: None,
            objects: Objects::new(),
            connections: None,
        }
    }
}

impl<C: FormatConvert> NodeLoaderCommon for FbxSceneLoader<C> {
    type Target = FbxScene<C::ImageResult>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(FbxScene {
            fbx_header_extension: try!(self.fbx_header_extension.ok_or(Error::UnclassifiedCritical("Required node `FbxHeaderExtension` not found".to_owned()))),
            objects: self.objects,
            connections: try!(self.connections.ok_or(Error::UnclassifiedCritical("Required node `Connections` not found".to_owned()))),
        })
    }
}

impl<R: Read, C: FormatConvert> NodeLoader<R> for FbxSceneLoader<C> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, .. } = node_info;
        match name.as_ref() {
            "FBXHeaderExtension" => {
                self.fbx_header_extension = Some(try!(FbxHeaderExtensionLoader::new().load(reader)));
            },
            "Definitions" => {
                self.definitions = Some(try!(DefinitionsLoader::new().load(reader)));
            },
            "Objects" => {
                let defs = try!(self.definitions.as_mut().ok_or(Error::UnclassifiedCritical("`Definitions` is required before `Objects` node".to_owned())));
                try!(ObjectsLoader::new(&mut self.objects, defs, &mut self.converter).load(reader));
            },
            "Connections" => {
                self.connections = Some(try!(ConnectionsLoader::new().load(reader)));
            },
            _ => {
                warn!("Unknown node: `{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}

pub fn load_scene<R: Read, C: FormatConvert>(reader: &mut EventReader<R>, fbx_version: i32, converter: C) -> Result<FbxScene<C::ImageResult>> {
    FbxSceneLoader::new(fbx_version, converter).load(reader)
}
