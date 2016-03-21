//! Contains common stuff for node loaders.

use std::io::Read;
use fbx_binary_reader::{EventReader, FbxEvent, DelayedProperties};
use error::Result;


#[derive(Debug, Clone)]
pub struct RawNodeInfo {
    /// Node name.
    pub name: String,
    /// Node properties.
    pub properties: DelayedProperties,
}

pub trait NodeLoader<R: Read>: Sized {
    type Target;

    fn load(mut self, reader: &mut EventReader<R>) -> Result<Self::Target> {
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

    /// Executed on node end.
    ///
    /// This is user-defined function.
    fn on_finish(self) -> Result<Self::Target>;
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
