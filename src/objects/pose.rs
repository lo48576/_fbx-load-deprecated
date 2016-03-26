//! Contains `/Objects/Pose` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;

#[derive(Debug, Clone)]
pub struct Pose {
    pub id: i64,
    pub name: String,
    pub pose_nodes: Vec<PoseNode>
}

pub struct PoseLoader<'a> {
    //definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    nb_pose_nodes: Option<i32>,
    pose_nodes: Option<Vec<PoseNode>>,
}

impl<'a> PoseLoader<'a> {
    pub fn new(_definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        PoseLoader {
            //definitions: definitions,
            obj_props: obj_props,
            nb_pose_nodes: None,
            pose_nodes: None,
        }
    }
}

impl<'a> NodeLoaderCommon for PoseLoader<'a> {
    type Target = Option<Pose>;

    fn on_finish(self) -> Result<Self::Target> {
        let nb_pose_nodes = if let Some(nb_pose_nodes) = self.nb_pose_nodes {
            nb_pose_nodes as usize
        } else {
            error!("Required property not found for `/Objects/Pose`: `/Objects/Pose/NbPoseNodes` not found");
            return Ok(None);
        };
        if let Some(pose_nodes) = self.pose_nodes {
            if nb_pose_nodes != pose_nodes.len() {
                error!("Number of `Pose/PoseNode`(={}) should be equal to the number specified by `NbPoseNodes`(={})", pose_nodes.len(), nb_pose_nodes);
                // Should the object be discarded?
            }
            Ok(Some(Pose {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                pose_nodes: pose_nodes,
            }))
        } else {
            error!("`Pose/NbPoseNodes` node is required but it was invalid or not found");
            Ok(None)
        }
    }
}

impl<'a, R: Read> NodeLoader<R> for PoseLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Type" => {
                match properties.iter().next().and_then(|p| p.get_string()) {
                    Some("BindPose") => {},
                    Some(t) => {
                        warn!("Maybe unsupported type of `/Objects/Pose` node: type={}", t);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Pose/Type`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Pose` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Pose/Version`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "NbPoseNodes" => {
                if self.nb_pose_nodes.is_some() {
                    error!("`/Objects/Pose/NbPoseNodes` appears more than twice in the same `Pose` node");
                } else {
                    match properties.iter().next().and_then(|p| p.get_i32()) {
                        Some(val) => if val >= 0 {
                            self.nb_pose_nodes = Some(val);
                            self.pose_nodes = Some(Vec::with_capacity(val as usize));
                        } else {
                            error!("Invalid property at `/Objects/Pose/NbPoseNodes`: expected positive value, but got `{}`", val);
                        },
                        None => {
                            error!("Invalid property at `/Objects/Pose/NbPoseNodes`: type error");
                        },
                    }
                }
                try!(ignore_current_node(reader));
            },
            "PoseNode" => {
                if let Some(ref mut pose_nodes) = self.pose_nodes {
                    if let Some(pose_node) = try!(PoseNodeLoader::new().load(reader)) {
                        pose_nodes.push(pose_node);
                    }
                } else {
                    try!(ignore_current_node(reader));
                }
            },
            _ => {
                warn!("Unknown node: `/Objects/Pose/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct PoseNode {
    pub node: i64,
    pub matrix: [[f32; 4]; 4],
}

#[derive(Debug, Default)]
pub struct PoseNodeLoader {
    pub node: Option<i64>,
    pub matrix: Option<[[f32; 4]; 4]>,
}

impl PoseNodeLoader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl NodeLoaderCommon for PoseNodeLoader {
    type Target = Option<PoseNode>;

    fn on_finish(self) -> Result<Self::Target> {
        if_all_some!((
                node=self.node,
                matrix=self.matrix,
            ) {
                return Ok(Some(PoseNode {
                    node: node,
                    matrix: matrix,
                }))
            } else {
                error!("Required node not found for `/Objects/Pose/PoseNode`");
                return Ok(None);
            });
    }
}

impl<R: Read> NodeLoader<R> for PoseNodeLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Node" => {
                self.node = properties.iter().next().and_then(|p| p.get_i64());
            },
            "Matrix" => {
                self.matrix = properties.iter().next().and_then(|p| p.as_vec_f32().into_iter().find(|v| v.len() >= 16).map(|v| {
                    [
                        [v[0], v[1], v[2], v[3]],
                        [v[4], v[5], v[6], v[7]],
                        [v[8], v[9], v[10], v[11]],
                        [v[12], v[13], v[14], v[15]],
                    ]
                }));
            },
            _ => {
                error!("Unknown node: `/Objects/Pose/PoseNode/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
