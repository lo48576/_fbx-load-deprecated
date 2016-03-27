//! Contains `/Objects/Deformer(Skin)` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;


#[derive(Debug, Clone)]
pub struct Skin {
    pub id: i64,
    pub link_deform_accuracy: f64,
    pub skinning_type: SkinningType,
}

#[derive(Debug)]
pub struct SkinLoader<'a> {
    obj_props: &'a ObjectProperties<'a>,
    link_deform_accuracy: Option<f64>,
    skinning_type: Option<SkinningType>,
}

impl<'a> SkinLoader<'a> {
    pub fn new(_definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        SkinLoader {
            obj_props: obj_props,
            link_deform_accuracy: None,
            skinning_type: None,
        }
    }
}

impl<'a> NodeLoaderCommon for SkinLoader<'a> {
    type Target = Option<Skin>;

    fn on_finish(self) -> Result<Self::Target> {
        if_all_some!{(
            link_deform_accuracy=self.link_deform_accuracy,
            skinning_type=self.skinning_type,
        ) {
            Ok(Some(Skin {
                id: self.obj_props.id,
                link_deform_accuracy: link_deform_accuracy,
                skinning_type: skinning_type,
            }))
        } else {
            error!("Required property not found for `/Objects/Deformer(Skin)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for SkinLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(101) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Deformer(Skin)` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Deformer(Skin)/Version`: type error");
                    },
                }
            },
            "Link_DeformAcuracy" => {
                self.link_deform_accuracy = properties.iter().next().and_then(|p| p.as_f64());
            },
            "SkinningType" => {
                self.skinning_type = properties.iter().next().and_then(|p| p.get_string()).and_then(SkinningType::from_str);
            },
            _ => {
                warn!("Unknown node: `/Objects/Deformer(Skin)/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}

/// Skinning type.
///
/// See [Help: FbxSkin Class
/// Reference](help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_class_fbx_skin_html#ad5d0e87f61ba99c47a539492df7917a1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinningType {
    Rigid,
    Linear,
    DualQuaternion,
    Blend,
}

impl SkinningType {
    pub fn from_str<T: AsRef<str>>(val: T) -> Option<Self> {
        match val.as_ref() {
            "Rigid" => Some(SkinningType::Rigid),
            "Linear" => Some(SkinningType::Linear),
            "DualQuaternion" => Some(SkinningType::DualQuaternion),
            "Blend" => Some(SkinningType::Blend),
            _ => None,
        }
    }
}
