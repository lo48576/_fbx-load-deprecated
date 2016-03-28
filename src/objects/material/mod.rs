//! Contains `/Objects/Material` node-related stuff.

pub use self::shading_parameters::{ShadingParameters, LambertParameters, PhongParameters};

use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader};

mod shading_parameters;


#[derive(Debug, Clone)]
pub struct Material {
    pub id: i64,
    pub name: String,
    pub shading_model: String,
    pub multi_layer: bool,
    pub shading_parameters: ShadingParameters,
}

#[derive(Debug)]
pub struct MaterialLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    properties: Option<GenericProperties>,
    shading_model: Option<String>,
    multi_layer: Option<bool>,
}

impl<'a> MaterialLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        MaterialLoader {
            definitions: definitions,
            obj_props: obj_props,
            properties: None,
            shading_model: None,
            multi_layer: None,
        }
    }
}

impl<'a> NodeLoaderCommon for MaterialLoader<'a> {
    type Target = Option<Material>;

    fn on_finish(mut self) -> Result<Self::Target> {
        if_all_some!{(
            shading_model=self.shading_model,
            multi_layer=self.multi_layer,
        ) {
            let shading_parameters = ShadingParameters::from_node_properties(&shading_model, &mut self.properties, &self.definitions.templates);
            Ok(Some(Material {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                shading_model: shading_model,
                multi_layer: multi_layer,
                shading_parameters: shading_parameters,
            }))
        } else {
            error!("Required property not found for `/Objects/Material`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for MaterialLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(102) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Material` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Material/Version`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "ShadingModel" => {
                self.shading_model = properties.iter().next().and_then(|p| p.get_string().map(|v| v.to_owned()));
                try!(ignore_current_node(reader));
            },
            "MultiLayer" => {
                self.multi_layer = properties.iter().next().and_then(|p| p.as_i64()).map(|v| v != 0);
                try!(ignore_current_node(reader));
            },
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            _ => {
                warn!("Unknown node: `/Objects/Material/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
