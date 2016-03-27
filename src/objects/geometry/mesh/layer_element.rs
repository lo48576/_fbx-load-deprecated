//! Contains structs for layer elements.

use std::io::Read;
use fbx_binary_reader::{EventReader, DelayedProperties};
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};


/// Mapping mode of layer element.
///
/// Note that,
///
/// - "a control point" means a vertex (x, y, z),
/// - "a polygon vertex" means an index to control point (in other words, vertex index),
/// - and "a polygon" means group of polygon vertices.
///
/// For detail of these words, see "Detailed Description" section of [Help: FbxMesh Class
/// Reference](http://help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_class_fbx_mesh_html#details).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MappingMode {
    /// The mapping is undetermined.
    None,
    /// One mapping coordinate for each "control point".
    // vertices[i] => map_elem[i]
    ByControlPoint,
    /// One mapping coordinate for each "polygon vertex".
    // vertices[polygon_vertex_index[i]] => map_elem[i]
    ByPolygonVertex,
    /// One mapping coordinate for each "polygon".
    // polygon[i] => map_elem[i]
    ByPolygon,
    /// "One mapping coordinate for each unique edge in the mesh."
    ///
    /// Quoted from [Help: FbxLayerElement Class
    /// Reference](http://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/class_fbx_layer_element.html#a865a00ff562c164136919bf777abb5e8).
    ByEdge,
    /// Only one mapping coordinate for the whole surface.
    // mesh => map_elem[i]
    AllSame,
}

impl MappingMode {
    /// Get a mapping mode value from the property value of a `MappingInformationType` node.
    pub fn from_string(name: &str) -> Option<MappingMode> {
        Some(match name {
            "ByControlPoint" | "ByVertex" | "ByVertice" => MappingMode::ByControlPoint,
            "ByPolygonVertex" => MappingMode::ByPolygonVertex,
            "ByPolygon" => MappingMode::ByPolygon,
            "ByEdge" => MappingMode::ByEdge,
            "AllSame" => MappingMode::AllSame,
            val => {
                error!("Invalid property at `/Objects/Geometry(Mesh)/LayerElement*/MappingInformationType`: unsupported value `{}`", val);
                return None;
            },
        })
    }

    pub fn from_node_properties(properties: &DelayedProperties) -> Option<MappingMode> {
        if let Some(name) = properties.iter().next().and_then(|p| p.get_string()) {
            Self::from_string(name)
        } else {
            error!("Invalid property at `/Objects/Geometry(Mesh)/LayerElement*/MappingInformationType`: not found or type error");
            None
        }
    }
}

/// Reference mode of layer elements.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceMode {
    /// No indices is required to use vertices.
    // layer_elem[i] => map_elem[i]
    Direct,
    /// The layer element has indices.
    // layer_elem[layer_elem_index[i]] => map_elem[i]
    IndexToDirect(Vec<u32>),
}

impl ReferenceMode {
    fn from_type_and_indices(t: ReferenceModeType, indices: Option<Vec<u32>>) -> Option<Self> {
        match t {
            ReferenceModeType::Direct => Some(ReferenceMode::Direct),
            ReferenceModeType::IndexToDirect => indices.map(ReferenceMode::IndexToDirect),
        }
    }

    pub fn has_indices(&self) -> bool {
        *self != ReferenceMode::Direct
    }
}

#[derive(Debug, Clone)]
pub struct LayerElement<T: Copy> {
    pub channel: i32,
    pub name: String,
    pub mapping_mode: MappingMode,
    pub reference_mode: ReferenceMode,
    pub data: Option<Vec<T>>,
}

#[derive(Debug)]
pub enum ReferenceModeType {
    Direct,
    IndexToDirect,
}

pub trait LoadAsLayerElementElement: Copy {
    fn node_properties_to_elements_array(properties: &DelayedProperties) -> Option<Vec<Self>>;
}

impl LoadAsLayerElementElement for () {
    fn node_properties_to_elements_array(_properties: &DelayedProperties) -> Option<Vec<()>> {
        None
    }
}

impl LoadAsLayerElementElement for [f32; 2] {
    fn node_properties_to_elements_array(properties: &DelayedProperties) -> Option<Vec<[f32; 2]>> {
        properties.iter().next().and_then(|p| p.as_vec_f32()
            .into_iter().find(|v| v.len() > 0) // Prevent `slice::chunks()` from panicking.
            .map(|vec| {
                let len = vec.len() / 2;
                vec.chunks(2).take(len).map(|e| [e[0], e[1]]).collect()
            }))
    }
}

impl LoadAsLayerElementElement for [f32; 3] {
    fn node_properties_to_elements_array(properties: &DelayedProperties) -> Option<Vec<[f32; 3]>> {
        properties.iter().next().and_then(|p| p.as_vec_f32()
            .into_iter().find(|v| v.len() > 0) // Prevent `slice::chunks()` from panicking.
            .map(|vec| {
                let len = vec.len() / 3;
                vec.chunks(3).take(len).map(|e| [e[0], e[1], e[2]]).collect()
            }))
    }
}

#[derive(Debug)]
pub struct LayerElementLoader<'a, T: LoadAsLayerElementElement> {
    pub data_node_name: &'a str,
    pub index_node_name: &'a str,
    pub channel: i32,
    pub name: Option<String>,
    pub mapping_mode: Option<MappingMode>,
    pub reference_mode: Option<ReferenceModeType>,
    pub data: Option<Vec<T>>,
    pub index: Option<Vec<u32>>,
}

impl<'a, T: LoadAsLayerElementElement> LayerElementLoader<'a, T> {
    pub fn from_node_properties(properties: &DelayedProperties, data_node_name: &'a str, index_node_name: &'a str) -> Option<Self> {
        if let Some(channel) = properties.iter().next().and_then(|p| p.get_i32()) {
            Some(Self::new(channel, data_node_name, index_node_name))
        } else {
            error!("Invalid property at `/Objects/Geometry(Mesh)/LayerElement*`: not found or type error");
            None
        }
    }

    pub fn new(channel: i32, data_node_name: &'a str, index_node_name: &'a str) -> Self {
        LayerElementLoader {
            data_node_name: data_node_name,
            index_node_name: index_node_name,
            channel: channel,
            name: None,
            mapping_mode: None,
            reference_mode: None,
            data: None,
            index: None,
        }
    }
}

impl<'a, T: LoadAsLayerElementElement> NodeLoaderCommon for LayerElementLoader<'a, T> {
    type Target = Option<LayerElement<T>>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let index = self.index.take();
        if_all_some!{(
            name=self.name,
            mapping_mode=self.mapping_mode,
            reference_mode=self.reference_mode.and_then(|t| ReferenceMode::from_type_and_indices(t, index)),
        ) {
            Ok(Some(LayerElement {
                channel: self.channel,
                name: name,
                mapping_mode: mapping_mode,
                reference_mode: reference_mode,
                data: self.data,
            }))
        } else {
            error!("Required property not found for `/Objects/Geometry(Mesh)/LayerElement*`");
            Ok(None)
        }}
    }
}

impl<'a, T: LoadAsLayerElementElement, R: Read> NodeLoader<R> for LayerElementLoader<'a, T> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(101...102) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Geometry(Mesh)/LayerElement*` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Geometry(Mesh)/LayerElement*/Version`: type error");
                    },
                }
            },
            "Name" => {
                self.name = properties.iter().next().and_then(|p| p.get_string()).map(|v| v.to_owned());
            },
            "MappingInformationType" => {
                self.mapping_mode = MappingMode::from_node_properties(&properties);
            },
            "ReferenceInformationType" => match properties.iter().next().and_then(|p| p.get_string()) {
                Some("Direct") => self.reference_mode = Some(ReferenceModeType::Direct),
                // In FBX 6.0 and higher, `Index` is replaced with `IndexToDirect`.
                // For detail, see
                // [Help: FbxLayerElement Class
                // Reference](http://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/class_fbx_layer_element.html#a445e03e8b14d7132b6f88dc6250a3394).
                Some("IndexToDirect") | Some("Index") => self.reference_mode = Some(ReferenceModeType::IndexToDirect),
                Some(val) => {
                    error!("Invalid property at `/Objects/Geometry(Mesh)/LayerElement*/ReferenceInformationType`: unsupported value `{}`", val);
                },
                None => {
                    error!("Invalid property at `/Objects/Geometry(Mesh)/LayerElement*/ReferenceInformationType`: not found or type error");
                },
            },
            _ if name == self.data_node_name => {
                self.data = T::node_properties_to_elements_array(&properties);
            },
            _ if name == self.index_node_name => {
                self.index = properties.iter().next().and_then(|p| p.extract_vec_i32().ok().map(|v| v.into_iter().map(|v| v as u32).collect()));
            },
            "NormalsW" => {}, // TODO: `NormalsW` may have euclidean norms of normals.
            _ => {
                warn!("Unknown node: `/Objects/Geometry(Mesh)/LayerElement*/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
