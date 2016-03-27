//! Contains `/Objects` node-related stuff.

pub use self::collection::DisplayLayer;
pub use self::deformer::{BlendShape, Skin, SkinningType};
pub use self::geometry::{Mesh, Shape, VertexIndex, MappingMode, ReferenceMode, LayerElement};
pub use self::material::{Material, ShadingParameters};
pub use self::model::{CullingType, Model};
pub use self::node_attribute::{LimbNodeAttribute, NullNodeAttribute, NodeAttributeType, NullNodeLook};
pub use self::pose::{Pose, PoseNode};
pub use self::texture::{Texture, BlendMode, WrapMode};
pub use self::video::Video;

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::io::Read;
use fbx_binary_reader::EventReader;
use fnv::FnvHasher;
use definitions::Definitions;
use error::Result;
use node_loader::{FormatConvert, NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use self::collection::{CollectionExclusive, CollectionExclusiveLoader};
use self::deformer::{Deformer, DeformerLoader};
use self::geometry::{Geometry, GeometryLoader};
use self::material::MaterialLoader;
use self::model::ModelLoader;
use self::node_attribute::{NodeAttribute, NodeAttributeLoader};
use self::pose::PoseLoader;
use self::properties::ObjectProperties;
use self::texture::TextureLoader;
use self::video::VideoLoader;

#[macro_use]
mod macros {
    macro_rules! if_all_some {
        // Specialize for single variable.
        {($v:ident=$e:expr) $some_block:block else $none_block:block} => (
            if let Some($v) = $e $some_block else $none_block
        );
        // Specialize for single variable with trailing comma.
        {($v:ident=$e:expr,) $some_block:block else $none_block:block} => (
            if let Some($v) = $e $some_block else $none_block
        );
        // Comma-separated multiple variables.
        {($($v:ident=$e:expr),+) $some_block:block else $none_block:block} => (
            if let ($(Some($v)),+) = ($($e),+) $some_block else $none_block
        );
        // Allow trailing comma.
        {($($v:ident=$e:expr),+,) $some_block:block else $none_block:block} => (
            if let ($(Some($v)),+) = ($($e),+) $some_block else $none_block
        );
    }
}

pub mod collection;
pub mod deformer;
pub mod geometry;
pub mod material;
pub mod model;
pub mod node_attribute;
pub mod pose;
pub mod properties;
pub mod texture;
pub mod video;


pub type ObjectsMap<V> = HashMap<i64, V, BuildHasherDefault<FnvHasher>>;

#[derive(Debug, Default, Clone)]
pub struct Objects<I: Clone> {
    pub unknown: ObjectsMap<UnknownObject>,
    pub blend_shapes: ObjectsMap<BlendShape>,
    pub display_layers: ObjectsMap<DisplayLayer>,
    pub geometry_meshes: ObjectsMap<Mesh>,
    pub geometry_shapes: ObjectsMap<Shape>,
    pub materials: ObjectsMap<Material>,
    pub model_limb_nodes: ObjectsMap<Model>,
    pub model_meshes: ObjectsMap<Model>,
    pub model_nulls: ObjectsMap<Model>,
    pub node_attribute_limb_nodes: ObjectsMap<LimbNodeAttribute>,
    pub node_attribute_nulls: ObjectsMap<NullNodeAttribute>,
    pub poses: ObjectsMap<Pose>,
    pub skins: ObjectsMap<Skin>,
    pub textures: ObjectsMap<Texture>,
    pub videos: ObjectsMap<Video<I>>,
}

impl<I: Clone> Objects<I> {
    pub fn new() -> Self {
        // TODO: It doesn't seem rustc-1.7.0 work correctly, `Default::default()` cannot compile
        //       (> error: the trait `core::default::Default` is not implemented for the type `I` [E0277]).
        //       See [#[derive] is too conservative with field trait bounds · Issue #26925 ·
        //       rust-lang/rust](https://github.com/rust-lang/rust/issues/26925).
        //Default::default()
        Objects {
            unknown: Default::default(),
            blend_shapes: Default::default(),
            display_layers: Default::default(),
            geometry_meshes: Default::default(),
            geometry_shapes: Default::default(),
            materials: Default::default(),
            model_limb_nodes: Default::default(),
            model_meshes: Default::default(),
            model_nulls: Default::default(),
            node_attribute_limb_nodes: Default::default(),
            node_attribute_nulls: Default::default(),
            poses: Default::default(),
            skins: Default::default(),
            textures: Default::default(),
            videos: Default::default(),
        }
    }
}

macro_rules! implement_method_for_object {
    ($plural:ident, $t:ty, $add_method:ident) => (
        impl<I: Clone> Objects<I> {
            pub fn $add_method(&mut self, obj: $t) {
                self.$plural.insert(obj.id, obj);
            }
        }
    )
}
implement_method_for_object!(unknown, UnknownObject, add_unknown);
implement_method_for_object!(blend_shapes, BlendShape, add_blend_shape);
implement_method_for_object!(display_layers, DisplayLayer, add_display_layer);
implement_method_for_object!(geometry_meshes, Mesh, add_geometry_mesh);
implement_method_for_object!(geometry_shapes, Shape, add_geometry_shape);
implement_method_for_object!(materials, Material, add_material);
implement_method_for_object!(model_limb_nodes, Model, add_model_limb_node);
implement_method_for_object!(model_meshes, Model, add_model_mesh);
implement_method_for_object!(model_nulls, Model, add_model_null);
implement_method_for_object!(node_attribute_nulls, NullNodeAttribute, add_node_attribute_null);
implement_method_for_object!(node_attribute_limb_nodes, LimbNodeAttribute, add_node_attribute_limb_node);
implement_method_for_object!(poses, Pose, add_pose);
implement_method_for_object!(skins, Skin, add_skin);
implement_method_for_object!(textures, Texture, add_texture);
implement_method_for_object!(videos, Video<I>, add_video);

#[derive(Debug)]
pub struct ObjectsLoader<'a, C: 'a + FormatConvert> {
    objects: &'a mut Objects<C::ImageResult>,
    definitions: &'a Definitions,
    converter: &'a mut C,
}

impl<'a, C: 'a + FormatConvert> ObjectsLoader<'a, C> {
    pub fn new(objects: &'a mut Objects<C::ImageResult>, definitions: &'a Definitions, converter: &'a mut C) -> Self {
        ObjectsLoader {
            objects: objects,
            definitions: definitions,
            converter: converter,
        }
    }
}

impl<'a, C: FormatConvert> NodeLoaderCommon for ObjectsLoader<'a, C> {
    type Target = ();

    fn on_finish(self) -> Result<Self::Target> {
        Ok(())
    }
}

impl<'a, R: Read, C: FormatConvert> NodeLoader<R> for ObjectsLoader<'a, C> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        let obj_props = if let Some(val) = ObjectProperties::from_node_properties(properties.iter()) {
            val
        } else {
            try!(ignore_current_node(reader));
            return Ok(());
        };
        match name.as_ref() {
            "CollectionExclusive" => match try!(CollectionExclusiveLoader::new(self.definitions, &obj_props).load(reader)) {
                Some(CollectionExclusive::DisplayLayer(obj)) => self.objects.add_display_layer(obj),
                Some(CollectionExclusive::Unknown(obj)) => self.objects.add_unknown(obj),
                None => {},
            },
            "Deformer" => match try!(DeformerLoader::new(self.definitions, &obj_props).load(reader)) {
                Some(Deformer::BlendShape(obj)) => self.objects.add_blend_shape(obj),
                Some(Deformer::Skin(obj)) => self.objects.add_skin(obj),
                Some(Deformer::Unknown(obj)) => self.objects.add_unknown(obj),
                None => {},
            },
            "Geometry" => match try!(GeometryLoader::new(self.definitions, &obj_props).load(reader)) {
                Some(Geometry::Mesh(obj)) => self.objects.add_geometry_mesh(obj),
                Some(Geometry::Shape(obj)) => self.objects.add_geometry_shape(obj),
                Some(Geometry::Unknown(obj)) => self.objects.add_unknown(obj),
                None => {},
            },
            "Material" => if let Some(obj) = try!(MaterialLoader::new(self.definitions, &obj_props).load(reader)) {
                self.objects.add_material(obj);
            },
            "Model" => if let Some(obj) = try!(ModelLoader::new(self.definitions, &obj_props).load(reader)) {
                match obj_props.subclass {
                    "LimbNode" => self.objects.add_model_limb_node(obj),
                    "Mesh" => self.objects.add_model_mesh(obj),
                    "Null" => self.objects.add_model_null(obj),
                    val => {
                        warn!("Unknown subclass({}) for `/Objects/Model` node, treat as UnknownObject", val);
                        self.objects.add_unknown(UnknownObject::from_object_properties(&obj_props));
                    },
                }
            },
            "NodeAttribute" => match try!(NodeAttributeLoader::new(self.definitions, &obj_props).load(reader)) {
                Some(NodeAttribute::LimbNode(obj)) => self.objects.add_node_attribute_limb_node(obj),
                Some(NodeAttribute::Null(obj)) => self.objects.add_node_attribute_null(obj),
                Some(NodeAttribute::Unknown(obj)) => self.objects.add_unknown(obj),
                None => {},
            },
            "Pose" => if let Some(obj) = try!(PoseLoader::new(self.definitions, &obj_props).load(reader)) {
                self.objects.add_pose(obj);
            },
            "Texture" => if let Some(obj) = try!(TextureLoader::new(self.definitions, &obj_props).load(reader)) {
                self.objects.add_texture(obj);
            },
            "Video" => if let Some(obj) = try!(VideoLoader::new(self.definitions, &obj_props, self.converter).load(reader)) {
                self.objects.add_video(obj);
            },
            _ => {
                warn!("Unknown object node: `/Objects/{}`", name);
                self.objects.unknown.insert(obj_props.id, UnknownObject {
                    id: obj_props.id,
                    name: obj_props.name.to_owned(),
                    class: obj_props.class.to_owned(),
                    subclass: obj_props.subclass.to_owned(),
                });
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct UnknownObject {
    pub id: i64,
    pub name: String,
    pub class: String,
    pub subclass: String,
}

impl UnknownObject {
    pub fn from_object_properties<'a>(props: &ObjectProperties<'a>) -> Self {
        UnknownObject {
            id: props.id,
            name: props.name.to_owned(),
            class: props.class.to_owned(),
            subclass: props.subclass.to_owned(),
        }
    }
}
