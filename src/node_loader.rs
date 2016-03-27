//! Contains common stuff for node loaders.

use std::io::Read;
use std::path::Path;
use fbx_binary_reader::{EventReader, FbxEvent, DelayedProperties};
use error::Result;


#[derive(Debug, Clone)]
pub struct RawNodeInfo {
    /// Node name.
    pub name: String,
    /// Node properties.
    pub properties: DelayedProperties,
}

pub trait NodeLoaderCommon: Sized {
    type Target;

    /// Executed on node end.
    ///
    /// This is user-defined function.
    fn on_finish(self) -> Result<Self::Target>;
}

pub trait NodeLoader<R: Read>: NodeLoaderCommon {
    fn load(mut self, reader: &mut EventReader<R>) -> Result<<Self as NodeLoaderCommon>::Target> {
        loop {
            match try!(reader.next()) {
                FbxEvent::StartFbx(_) => unreachable!(),
                FbxEvent::StartNode { name, properties } => {
                    try!(self.on_child_node(reader, RawNodeInfo { name: name, properties: properties }));
                },
                FbxEvent::EndFbx | FbxEvent::EndNode => {
                    return self.on_finish();
                },
            }
        }
    }

    /// Executed for each children
    ///
    /// This is user-defined function.
    fn on_child_node(&mut self, reader: &mut EventReader<R>, _node_info: RawNodeInfo) -> Result<()> {
        try!(ignore_current_node(reader));
        Ok(())
    }
}

pub fn ignore_current_node<R: Read>(reader: &mut EventReader<R>) -> Result<()> {
    let mut level = 1_usize;
    loop {
        match try!(reader.next()) {
            FbxEvent::StartNode { .. } => {
                level += 1;
            },
            FbxEvent::EndNode => {
                level -= 1;
                if level == 0 {
                    return Ok(());
                }
            },
            FbxEvent::EndFbx => {
                level -= 1;
                assert_eq!(level, 0);
                return Ok(());
            },
            _ => {},
        }
    }
}

pub trait FormatConvert {
    type ImageResult: Clone;

    fn binary_to_image(&mut self, binary: &[u8], path: &Path) -> Self::ImageResult;
}

impl<'a, T: FormatConvert> FormatConvert for &'a mut T {
    type ImageResult = <T as FormatConvert>::ImageResult;

    fn binary_to_image(&mut self, binary: &[u8], path: &Path) -> Self::ImageResult {
        (**self).binary_to_image(binary, path)
    }
}
