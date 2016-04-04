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
    NotTriangulated(Vec<i32>),
    Triangulated(Vec<u32>),
}

struct TriangulationInfo {
    pub tri_vertex_index: Vec<u32>,
    pub tri_pvi_to_src_pvi: Vec<u32>,
    pub tri_poly_to_src_poly: Vec<u32>,
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

impl Mesh {
    /// Triangulates all polygons in the mesh with the given triangulation function.
    ///
    /// This function modifies `polygon_vertex_index` and layer elements, but doesn't change
    /// `vertices`.
    pub fn triangulate<F>(&mut self, triangulator: F)
        where F: Fn(&[[f32; 3]], &[u32], &mut Vec<u32>) -> u32
    {
        // Triangulate and update layer elements only when the vertex index (polygon vertices) is
        // not yet triangulated.
        if let Some(result) = self.triangulate_polygon_index(triangulator) {
            self.polygon_vertex_index = VertexIndex::Triangulated(result.tri_vertex_index);
            // Update layer elements in accordance with updated polygon vertices
            // `tri_vertex_index`.
            self.apply_triangulation_to_layer_elements(&result.tri_pvi_to_src_pvi, &result.tri_poly_to_src_poly);
        }
    }

    fn triangulate_polygon_index<F>(&self, triangulator: F) -> Option<TriangulationInfo>
        where F: Fn(&[[f32; 3]], &[u32], &mut Vec<u32>) -> u32
    {
        // Triangulate and update layer elements only when the vertex index (polygon vertices) is
        // not yet triangulated.
        let polygon_vertex_index = if let VertexIndex::NotTriangulated(ref polygon_vertex_index) = self.polygon_vertex_index {
            polygon_vertex_index
        } else {
            return None;
        };
        // Triangulation.

        // # Return values (of this method)
        // `tri_vertex_index`: Polygon vertex indeices of triangulated polygons.
        // `tri_pvi_to_src_pvi`: "Triangulated polygon vertex index to source polygon vertex index".
        //                       Array to convert polygon vertex (index) of triangulated polygons
        //                       to source polygon vertex.
        //                       Note that "polygon vertex" means "index to control point", and
        //                       "control point" means "vertex".
        //                       See [Help: FbxMesh Class
        //                       Reference](http://help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_class_fbx_mesh_html#details)
        //                       for these terms.
        // `tri_poly_to_src_poly`: Array to convert index of triangles to index of source polygons.
        //                         This will be necessary to modify layer elements with mapping
        //                         mode `ByPolygon`.
        let mut tri_vertex_index = vec![];
        let mut tri_pvi_to_src_pvi = vec![];
        let mut tri_poly_to_src_poly = vec![];

        // # Temporary variables
        // Iterator of source polygon vertex.
        let mut source_pv_iter = polygon_vertex_index.iter().enumerate();
        // Polygon vertices of a processing polygon.
        let mut current_polygon_pv = vec![];
        // Polygon-local (beginning from 0) indices of polygon vertex of a triangulated polygons.
        let mut tri_local_indices = Vec::with_capacity(polygon_vertex_index.len());
        // Current polygon index.
        let mut current_poly_index = 0;

        'all_indices: loop {
            current_polygon_pv.clear();
            tri_local_indices.clear();
            // Index of polygon vertex at the beginning of the current polygon.
            let start_pv_index;
            // Get single polygon.
            'getting_polygon: loop {
                if let Some((pv_index, &current_pv)) = source_pv_iter.next() {
                    if current_pv < 0 {
                        // This `pv_index` is the last polygon vertex of the current polygon.
                        current_polygon_pv.push(!current_pv as u32);
                        start_pv_index = pv_index - (current_polygon_pv.len() - 1);
                        break 'getting_polygon;
                    } else {
                        current_polygon_pv.push(current_pv as u32);
                    }
                } else {
                    // No more valid polygons to triangulate.
                    if !current_polygon_pv.is_empty() {
                        warn!("Polygon vertex index didn't end with negtive number");
                    }
                    break 'all_indices;
                }
            }
            // Triangulate the gotten polygon.
            let num_of_triangles = triangulator(&self.vertices, &current_polygon_pv, &mut tri_local_indices);
            assert_eq!(num_of_triangles as usize * 3, tri_local_indices.len());

            tri_vertex_index.extend(tri_local_indices.iter().map(|&i| current_polygon_pv[i as usize]));
            tri_pvi_to_src_pvi.extend(tri_local_indices.iter().map(|&i| start_pv_index as u32 + i));
            {
                let tri_poly_to_src_poly_len = tri_poly_to_src_poly.len();
                tri_poly_to_src_poly.resize(tri_poly_to_src_poly_len + num_of_triangles as usize, current_poly_index);
            }
            current_poly_index += 1;
        }

        if tri_vertex_index.len() > ::std::u32::MAX as usize {
            // Not a bug, but unsupported data.
            panic!("Too many triangles in mesh (id={}, name=`{}`): length of vertex indices ({}) is over u32::MAX", self.id, self.name, tri_vertex_index.len());
        }
        assert_eq!(tri_vertex_index.len(), tri_pvi_to_src_pvi.len());
        assert_eq!(tri_vertex_index.len() % 3, 0);

        // Triangulation is done.
        Some(TriangulationInfo {
            tri_vertex_index: tri_vertex_index,
            tri_pvi_to_src_pvi: tri_pvi_to_src_pvi,
            tri_poly_to_src_poly: tri_poly_to_src_poly,
        })
    }

    fn apply_triangulation_to_layer_elements(&mut self, tri_pvi_to_src_pvi: &Vec<u32>, tri_poly_to_src_poly: &Vec<u32>) {
        update_layer_elements(&mut self.layer_element_materials, tri_pvi_to_src_pvi, tri_poly_to_src_poly);
        update_layer_elements(&mut self.layer_element_normals, tri_pvi_to_src_pvi, tri_poly_to_src_poly);
        update_layer_elements(&mut self.layer_element_uvs, tri_pvi_to_src_pvi, tri_poly_to_src_poly);
    }

    /// Returns "polygon vertex" (control point index) list of triangulated polygon.
    ///
    /// # Panics
    /// This function should be called on `triangulate()`d mesh.
    /// If the mesh is not yet triangulated, this function panics.
    pub fn triangulated_index_list(&self) -> &Vec<u32> {
        match self.polygon_vertex_index {
            VertexIndex::Triangulated(ref pvi) => &pvi,
            _ => panic!("`Mesh::get_expanded_triangles_list()` called on not triangulated mesh"),
        }
    }
}

fn update_layer_elements<'a, T, I>(layer_elements: I, tri_pvi_to_src_pvi: &Vec<u32>, tri_poly_to_src_poly: &Vec<u32>)
    where T: 'a + Copy,
          I: 'a + IntoIterator<Item = &'a mut LayerElement<T>>,
{
    for le in layer_elements.into_iter() {
        match le.mapping_mode {
            // None: No knowledge about the mapping mode.
            MappingMode::None |
            // ByControlPoint: Control point is not changed.
            MappingMode::ByControlPoint |
            // ByEdge: Edge-related feature is not supported by current `fbx_load` crate.
            MappingMode::ByEdge |
            // AllSame: No dependency on polygons.
            MappingMode::AllSame => {
                // Do nothing.
            },
            MappingMode::ByPolygonVertex => {
                // NOTE: Update can be more effective by changing reference mode from `Direct`
                //       to `IndexToDirect`, but this function doesn't do it (because the modes
                //       restriction for each layer element is unknown).
                match le.reference_mode {
                    ReferenceMode::Direct => {
                        // Update vertices.
                        if let Some(ref mut data) = le.data {
                            *data = tri_pvi_to_src_pvi.iter().map(|&i| data[i as usize]).collect();
                        }
                    },
                    ReferenceMode::IndexToDirect(ref mut indices) => {
                        // Update index.
                        *indices = tri_pvi_to_src_pvi.iter().map(|&src_pvi| indices[src_pvi as usize]).collect();
                    },
                }
            },
            MappingMode::ByPolygon => {
                match le.reference_mode {
                    ReferenceMode::Direct => {
                        if let Some(ref mut data) = le.data {
                            *data = tri_poly_to_src_poly.iter().map(|&i| data[i as usize]).collect();
                        }
                    },
                    ReferenceMode::IndexToDirect(ref mut indices) => {
                        *indices = tri_poly_to_src_poly.iter().map(|&src_poly| indices[src_poly as usize]).collect();
                    },
                }
            },
        }
    }
}

#[derive(Debug)]
pub struct MeshLoader<'a> {
    //definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    vertices: Option<Vec<[f32; 3]>>,
    polygon_vertex_index: Option<Vec<i32>>,
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
                            vec.chunks(3).take(len).map(|v| [v[0], v[1], v[2]]).collect()
                        }));
                try!(ignore_current_node(reader));
            },
            "PolygonVertexIndex" => {
                self.polygon_vertex_index = properties.iter().next().and_then(|p| p.extract_vec_i32().ok());
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
