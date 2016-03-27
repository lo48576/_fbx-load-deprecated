//! Contains `/Objects/Deformer(BlendShapeChannel)` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;


#[derive(Debug, Clone)]
pub struct BlendShapeChannel {
    pub id: i64,
    pub deform_percent: f64,
    pub full_weights: Vec<f32>,
}

#[derive(Debug)]
pub struct BlendShapeChannelLoader<'a> {
    obj_props: &'a ObjectProperties<'a>,
    deform_percent: Option<f64>,
    full_weights: Option<Vec<f32>>,
}

impl<'a> BlendShapeChannelLoader<'a> {
    pub fn new(_definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        BlendShapeChannelLoader {
            obj_props: obj_props,
            deform_percent: None,
            full_weights: None,
        }
    }
}

impl<'a> NodeLoaderCommon for BlendShapeChannelLoader<'a> {
    type Target = Option<BlendShapeChannel>;

    fn on_finish(self) -> Result<Self::Target> {
        if_all_some!{(
            full_weights=self.full_weights,
        ) {
            Ok(Some(BlendShapeChannel {
                id: self.obj_props.id,
                // Default value is 0.
                // See [Help: FbxBlendShapeChannel Class
                // Reference](http://help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_class_fbx_blend_shape_channel_html#a81e8c6b125b60687b414e3aa8f2bfc7a)
                // for detail.
                deform_percent: self.deform_percent.unwrap_or(0.0),
                full_weights: full_weights,
            }))
        } else {
            error!("Required property not found for `/Objects/Deformer(BlendShapeChannel)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for BlendShapeChannelLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Deformer(BlendShapeChannel)` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Deformer(BlendShapeChannel)/Version`: type error");
                    },
                }
            },
            // NOTE: `Properties70` may also have `DeformPercent`, but it always seems to have same
            //       value as `Deformer/Deformpercent`.
            "DeformPercent" => {
                self.deform_percent = properties.iter().next().and_then(|p| p.as_f64());
            },
            "FullWeights" => {
                self.full_weights = properties.iter().next().and_then(|p| p.into_vec_f32().ok());
            },
            "Properties70" => {},
            _ => {
                warn!("Unknown node: `/Objects/Deformer(BlendShapeChannel)/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
