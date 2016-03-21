///! Contains FBX Scene related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use error::Result;

#[derive(Debug, Clone)]
pub struct FbxScene;

pub fn load_scene<R: Read>(reader: &mut EventReader<R>, fbx_version: i32) -> Result<FbxScene> {
    unimplemented!()
}
