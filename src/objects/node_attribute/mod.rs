//! Contains `/Objects/NodeAttribute` node-related stuff.

pub use self::limb_node::LimbNodeAttribute;
pub use self::null::{NullNodeAttribute, NullNodeLook};

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::UnknownObject;
use objects::properties::ObjectProperties;
use self::limb_node::LimbNodeAttributeLoader;
use self::null::NullNodeAttributeLoader;

pub mod limb_node;
pub mod null;


#[derive(Debug, Clone)]
pub enum NodeAttribute {
    LimbNode(LimbNodeAttribute),
    Null(NullNodeAttribute),
    Unknown(UnknownObject),
}

#[derive(Debug)]
pub enum NodeAttributeLoader<'a> {
    LimbNode(LimbNodeAttributeLoader<'a>),
    Null(NullNodeAttributeLoader<'a>),
    Unknown(&'a ObjectProperties<'a>),
}

impl<'a> NodeAttributeLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        match obj_props.subclass {
            "LimbNode" => NodeAttributeLoader::LimbNode(LimbNodeAttributeLoader::new(definitions, obj_props)),
            "Null" => NodeAttributeLoader::Null(NullNodeAttributeLoader::new(definitions, obj_props)),
            val => {
                warn!("Unknown subclass({}) for `/Objects/CollectionExclusive`, treat as UnknownObject", val);
                NodeAttributeLoader::Unknown(obj_props)
            },
        }
    }
}

impl<'a> NodeLoaderCommon for NodeAttributeLoader<'a> {
    type Target = Option<NodeAttribute>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(match self {
            NodeAttributeLoader::LimbNode(loader) => try!(loader.on_finish()).map(NodeAttribute::LimbNode),
            NodeAttributeLoader::Null(loader) => try!(loader.on_finish()).map(NodeAttribute::Null),
            NodeAttributeLoader::Unknown(obj_props) => Some(NodeAttribute::Unknown(UnknownObject::from_object_properties(obj_props))),
        })
    }
}

impl<'a, R: Read> NodeLoader<R> for NodeAttributeLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        match *self {
            NodeAttributeLoader::LimbNode(ref mut loader) => loader.on_child_node(reader, node_info),
            NodeAttributeLoader::Null(ref mut loader) => loader.on_child_node(reader, node_info),
            NodeAttributeLoader::Unknown(_) => {
                try!(ignore_current_node(reader));
                Ok(())
            },
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NodeAttributeType {
    Unknown,
    Null,
    Marker,
    Skeleton,
    Mesh,
    Nurbs,
    Patch,
    Camera,
    CameraStereo,
    CameraSwitcher,
    Light,
    OpticalReference,
    OpticalMarker,
    NurbsCurve,
    TrimNurbsSurface,
    Boundary,
    NurbsSurface,
    Shape,
    LODGroup,
    SubDiv,
    CachedEffect,
    Line,
}

impl NodeAttributeType {
    pub fn from_string<S: AsRef<str>>(name: S) -> Self {
        match name.as_ref() {
            "Unknown" => NodeAttributeType::Unknown,
            "Null" => NodeAttributeType::Null,
            "Marker" => NodeAttributeType::Marker,
            "Skeleton" => NodeAttributeType::Skeleton,
            "Mesh" => NodeAttributeType::Mesh,
            "Nurbs" => NodeAttributeType::Nurbs,
            "Patch" => NodeAttributeType::Patch,
            "Camera" => NodeAttributeType::Camera,
            "CameraStereo" => NodeAttributeType::CameraStereo,
            "CameraSwitcher" => NodeAttributeType::CameraSwitcher,
            "Light" => NodeAttributeType::Light,
            "OpticalReference" => NodeAttributeType::OpticalReference,
            "OpticalMarker" => NodeAttributeType::OpticalMarker,
            "NurbsCurve" => NodeAttributeType::NurbsCurve,
            "TrimNurbsSurface" => NodeAttributeType::TrimNurbsSurface,
            "Boundary" => NodeAttributeType::Boundary,
            "NurbsSurface" => NodeAttributeType::NurbsSurface,
            "Shape" => NodeAttributeType::Shape,
            "LODGroup" => NodeAttributeType::LODGroup,
            "SubDiv" => NodeAttributeType::SubDiv,
            "CachedEffect" => NodeAttributeType::CachedEffect,
            "Line" => NodeAttributeType::Line,
            v => {
                error!("Invalid value (`{}`) as `/Objects/NodeAttribuete/TypeFlags`, treat as `unknown`", v);
                NodeAttributeType::Unknown
            },
        }
    }
}

impl Default for NodeAttributeType {
    fn default() -> Self {
        NodeAttributeType::Unknown
    }
}
