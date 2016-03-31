//! Contains `/Objects/Model` node-related stuff.

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader, OptionalProperties};


#[derive(Debug, Clone)]
pub struct Model {
    pub id: i64,
    pub name: String,
    pub shading: bool,
    pub culling: CullingType,
    pub axis_len: f64,
    pub show: bool,
    pub inherit_type: InheritType,
}

#[derive(Debug)]
pub struct ModelLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    properties: Option<GenericProperties>,
    shading: Option<bool>,
    culling: Option<CullingType>,
    axis_len: Option<f64>,
    show: Option<bool>,
    inherit_type: Option<InheritType>,
}

impl<'a> ModelLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        ModelLoader {
            definitions: definitions,
            obj_props: obj_props,
            properties: None,
            shading: None,
            culling: None,
            axis_len: None,
            show: None,
            inherit_type: None,
        }
    }
}

impl<'a> NodeLoaderCommon for ModelLoader<'a> {
    type Target = Option<Model>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let defaults = self.definitions.templates.templates.get(&("Model".to_owned(), "FbxNode".to_owned())).map(|t| &t.properties);
        let axis_len = self.properties.get_or_default(defaults, "AxisLen").and_then(|p| p.value.get_f64());
        let show = self.properties.get_or_default(defaults, "Show").and_then(|p| p.value.get_i64()).map(|v| v != 0);
        let inherit_type = self.properties.get_or_default(defaults, "InheritType").and_then(|p| p.value.get_i64()).and_then(InheritType::from_i64);
        // There still remains many properties to read. For more information, see [Help: FbxNode Class
        // Reference](http://help.autodesk.com/view/FBX/2016/ENU/?guid=__cpp_ref_class_fbx_node_html#pub-attribs).
        if_all_some!{(
            shading=self.shading,
            culling=self.culling,
            axis_len=axis_len,
            show=show,
            inherit_type=inherit_type,
        ) {
            Ok(Some(Model {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                shading: shading,
                culling: culling,
                axis_len: axis_len,
                show: show,
                inherit_type: inherit_type,
            }))
        } else {
            error!("Required property not found for `/Objects/Model({})`", self.obj_props.subclass);
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for ModelLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(232) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Model({})` node: ver={}", self.obj_props.subclass, v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Model({})/Version`: type error", self.obj_props.subclass);
                    },
                }
                try!(ignore_current_node(reader));
            },
            "Shading" => {
                self.shading = properties.iter().next().and_then(|p| p.get_bool());
                try!(ignore_current_node(reader));
            },
            "Culling" => {
                self.culling = properties.iter().next().and_then(|p| p.get_string().and_then(CullingType::from_str));
                try!(ignore_current_node(reader));
            },
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            _ => {
                warn!("Unknown node: `/Objects/Model({})/{}`", self.obj_props.subclass, name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}


#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CullingType {
    Off,
    Ccw,
    Cw,
}

impl CullingType {
    pub fn from_str<S: AsRef<str>>(s: S) -> Option<Self> {
        match s.as_ref() {
            "CullingOff" => Some(CullingType::Off),
            "CullingOnCCW" => Some(CullingType::Ccw),
            "CullingOnCW" => Some(CullingType::Cw),
            _ => None,
        }
    }
}

/// See [Help: FbxTransform Class
/// Reference](http://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/class_fbx_transform.html#a0affdd70d8df512d82fdb6a30112bf0c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InheritType {
    RrSs,
    RSrs,
    Rrs,
}

impl InheritType {
    pub fn from_i64(v: i64) -> Option<InheritType> {
        match v {
            0 => Some(InheritType::RrSs),
            1 => Some(InheritType::RSrs),
            2 => Some(InheritType::Rrs),
            _ => None,
        }
    }
}
