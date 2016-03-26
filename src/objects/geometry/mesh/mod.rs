//! Contains `/Objects/Geometry(Mesh)` node-related stuff.

pub use self::layer::Layer;
pub use self::layer_element::{MappingMode, ReferenceMode, LayerElement};

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use self::layer::LayerLoader;
use self::layer_element::LayerElementLoader;

mod layer;
mod layer_element;


#[derive(Debug, Clone)]
pub enum VertexIndex {
    NotTriangulated(Vec<u32>),
    Triangulated(Vec<u32>),
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub id: i64,
    pub name: String,
    pub vertices: Vec<[f32; 3]>,
    pub polygon_vertex_index: VertexIndex,
    pub layer_element_materials: Vec<LayerElement<()>>,
    pub layer_element_normals: Vec<LayerElement<[f32; 3]>>,
    pub layer_element_uvs: Vec<LayerElement<[f32; 2]>>,
    pub layers: Vec<Layer>,
}

#[derive(Debug)]
pub struct MeshLoader<'a> {
    //definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    vertices: Option<Vec<[f32; 3]>>,
    polygon_vertex_index: Option<Vec<u32>>,
    layer_element_materials: Vec<LayerElement<()>>,
    layer_element_normals: Vec<LayerElement<[f32; 3]>>,
    layer_element_uvs: Vec<LayerElement<[f32; 2]>>,
    layers: Vec<Layer>,
}

impl<'a> MeshLoader<'a> {
    pub fn new(_definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        MeshLoader {
            //definitions: definitions,
            obj_props: obj_props,
            vertices: None,
            polygon_vertex_index: None,
            layer_element_materials: Default::default(),
            layer_element_normals: Default::default(),
            layer_element_uvs: Default::default(),
            layers: Default::default(),
        }
    }
}

impl<'a> NodeLoaderCommon for MeshLoader<'a> {
    type Target = Option<Mesh>;

    fn on_finish(self) -> Result<Self::Target> {
        if_all_some!{(
            vertices=self.vertices,
            polygon_vertex_index=self.polygon_vertex_index,
        ) {
            Ok(Some(Mesh {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                vertices: vertices,
                polygon_vertex_index: VertexIndex::NotTriangulated(polygon_vertex_index),
                layer_element_materials: self.layer_element_materials,
                layer_element_normals: self.layer_element_normals,
                layer_element_uvs: self.layer_element_uvs,
                layers: self.layers,
            }))
        } else {
            error!("Required property not found for `/Objects/Geometry(Mesh)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for MeshLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Vertices" => {
                self.vertices = properties.iter().next()
                    .and_then(|p| p.as_vec_f32()
                        .into_iter().find(|vec| vec.len() > 0) // Prevent vec.chunks() from panicking.
                        .map(|vec| {
                            let len = vec.len() / 3;
                            vec.chunks(len).map(|v| [v[0], v[1], v[2]]).collect()
                        }));
                try!(ignore_current_node(reader));
            },
            "PolygonVertexIndex" => {
                self.polygon_vertex_index = properties.iter().next().and_then(|p| p.get_vec_i32().map(|v| v.into_iter().map(|&v| v as u32).collect()));
                try!(ignore_current_node(reader));
            },
            "GeometryVersion" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(124) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Geometry(Mesh)` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Geometry(Mesh)/Layer/GeometryVersion`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "LayerElementMaterial" => if let Some(loader) = LayerElementLoader::<()>::from_node_properties(&properties, "", "Materials") {
                if let Some(layer_elem) = try!(loader.load(reader)) {
                    self.layer_element_materials.push(layer_elem);
                }
            } else {
                try!(ignore_current_node(reader));
            },
            "LayerElementNormal" => if let Some(loader) = LayerElementLoader::<[f32; 3]>::from_node_properties(&properties, "Normals", "NormalsIndex") {
                if let Some(layer_elem) = try!(loader.load(reader)) {
                    self.layer_element_normals.push(layer_elem);
                }
            } else {
                try!(ignore_current_node(reader));
            },
            "LayerElementUV" => if let Some(loader) = LayerElementLoader::<[f32; 2]>::from_node_properties(&properties, "UV", "UVIndex") {
                if let Some(layer_elem) = try!(loader.load(reader)) {
                    self.layer_element_uvs.push(layer_elem);
                }
            } else {
                try!(ignore_current_node(reader));
            },
            "Layer" => if let Some(loader) = LayerLoader::from_node_properties(&properties) {
                if let Some(layer) = try!(loader.load(reader)) {
                    self.layers.push(layer);
                }
            } else {
                try!(ignore_current_node(reader));
            },
            "Edges" => {
                try!(ignore_current_node(reader));
            },
            _ => {
                warn!("Unknown node: `/Objects/Geometry(Mesh)/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
