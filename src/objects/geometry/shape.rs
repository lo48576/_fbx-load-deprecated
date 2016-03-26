//! Contains `/Objects/Geometry(Shape)` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;


#[derive(Debug, Clone)]
pub struct Shape {
    pub id: i64,
    pub name: String,
    // Indices of control points of a target mesh.
    pub indices: Vec<u32>,
    pub vertices: Vec<[f32; 3]>,
    pub normals: Option<Vec<[f32; 3]>>,
}

#[derive(Debug)]
pub struct ShapeLoader<'a> {
    obj_props: &'a ObjectProperties<'a>,
    indices: Option<Vec<u32>>,
    vertices: Option<Vec<[f32; 3]>>,
    normals: Option<Vec<[f32; 3]>>,
}

impl<'a> ShapeLoader<'a> {
    pub fn new(_definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        ShapeLoader {
            //definitions: definitions,
            obj_props: obj_props,
            indices: None,
            vertices: None,
            normals: None,
        }
    }
}

impl<'a> NodeLoaderCommon for ShapeLoader<'a> {
    type Target = Option<Shape>;

    fn on_finish(self) -> Result<Self::Target> {
        if_all_some!{(
            indices=self.indices,
            vertices=self.vertices,
        ) {
            Ok(Some(Shape {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                indices: indices,
                vertices: vertices,
                normals: self.normals,
            }))
        } else {
            error!("Required property not found for `/Objects/Geometry(Shape)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for ShapeLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Geometry(Shape)` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Geometry(Shape)/Version`: type error");
                    },
                }
            },
            "Indexes" => {
                self.indices = properties.iter().next().and_then(|p| p.get_vec_i32().map(|v| v.into_iter().map(|&v| v as u32).collect()));
            },
            "Vertices" => {
                self.vertices = properties.iter().next()
                    .and_then(|p| p.as_vec_f32())
                    .into_iter().find(|vec| vec.len() > 0) // Prevent vec.chunks() from panicking.
                    .map(|vec| {
                        let len = vec.len() / 3;
                        vec.chunks(3).take(len).map(|v| [v[0], v[1], v[2]]).collect()
                    });
            },
            "Normals" => {
                self.normals = properties.iter().next()
                    .and_then(|p| p.as_vec_f32())
                    .into_iter().find(|vec| vec.len() > 0) // Prevent vec.chunks() from panicking.
                    .map(|vec| {
                        let len = vec.len() / 3;
                        vec.chunks(3).take(len).map(|v| [v[0], v[1], v[2]]).collect()
                    });
            },
            _ => {
                warn!("Unknown node: `/Objects/Geometry(Shape)/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
