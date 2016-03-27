//! Contains `/Objects/Deformer(Cluster)` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;


#[derive(Debug, Clone)]
pub struct Cluster {
    pub id: i64,
    pub user_id: String,
    pub user_data: String,
    pub indices: Vec<u32>,
    pub weights: Vec<f32>,
    pub transform: [[f32; 4]; 4],
    pub transform_link: [[f32; 4]; 4],
}

#[derive(Debug)]
pub struct ClusterLoader<'a> {
    obj_props: &'a ObjectProperties<'a>,
    user_data: Option<(String, String)>,
    indices: Option<Vec<u32>>,
    weights: Option<Vec<f32>>,
    transform: Option<[[f32; 4]; 4]>,
    transform_link: Option<[[f32; 4]; 4]>,
}

impl<'a> ClusterLoader<'a> {
    pub fn new(_definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        ClusterLoader {
            obj_props: obj_props,
            user_data: None,
            indices: None,
            weights: None,
            transform: None,
            transform_link: None,
        }
    }
}

impl<'a> NodeLoaderCommon for ClusterLoader<'a> {
    type Target = Option<Cluster>;

    fn on_finish(self) -> Result<Self::Target> {
        if self.indices.as_ref().map(|v| v.len()) != self.weights.as_ref().map(|v| v.len()) {
            error!("Inconsistent data at `/Objects/Deformer(Cluster)` node: Number of elements in `Indexes` and `Weights` not matched");
            return Ok(None);
        }
        // Note that `Indexes` and `Weights` node might not exist.
        if_all_some!{(
            (user_id, user_data)=self.user_data,
            transform=self.transform,
            transform_link=self.transform_link,
        ) {
            Ok(Some(Cluster {
                id: self.obj_props.id,
                user_id: user_id,
                user_data: user_data,
                indices: self.indices.unwrap_or_default(),
                weights: self.weights.unwrap_or_default(),
                transform: transform,
                transform_link: transform_link,
            }))
        } else {
            error!("Required property not found for `/Objects/Deformer(Cluster)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for ClusterLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Deformer(Deformer)` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Deformer(Deformer)/Version`: type error");
                    },
                }
            },
            "UserData" => {
                let mut iter = properties.iter();
                let first = iter.next().and_then(|p| p.get_string());
                let second = iter.next().and_then(|p| p.get_string());
                if_all_some!{(first=first, second=second) {
                    self.user_data = Some((first.to_owned(), second.to_owned()));
                }}
            },
            "Indexes" => {
                self.indices = properties.iter().next().and_then(|p| p.extract_vec_i32().ok().map(|v| v.into_iter().map(|v| v as u32).collect()));
            },
            "Weights" => {
                self.weights = properties.iter().next().and_then(|p| p.into_vec_f32().ok());
            },
            "Transform" => {
                self.transform = properties.iter().next().and_then(|p| p.as_vec_f32().into_iter().find(|v| v.len() >= 16).map(|v| {
                    [
                        [v[0], v[1], v[2], v[3]],
                        [v[4], v[5], v[6], v[7]],
                        [v[8], v[9], v[10], v[11]],
                        [v[12], v[13], v[14], v[15]],
                    ]
                }));
            },
            "TransformLink" => {
                self.transform_link = properties.iter().next().and_then(|p| p.as_vec_f32().into_iter().find(|v| v.len() >= 16).map(|v| {
                    [
                        [v[0], v[1], v[2], v[3]],
                        [v[4], v[5], v[6], v[7]],
                        [v[8], v[9], v[10], v[11]],
                        [v[12], v[13], v[14], v[15]],
                    ]
                }));
            },
            _ => {
                warn!("Unknown node: `/Objects/Deformer(Cluster)/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
