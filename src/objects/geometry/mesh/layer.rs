use std::io::Read;
use fbx_binary_reader::{EventReader, DelayedProperties};
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};


#[derive(Debug, Clone)]
pub struct Layer {
    pub channel: i32,
    pub material: Vec<i32>,
    pub normal: Vec<i32>,
    pub uv: Vec<i32>,
    //pub color: Vec<i32>, // LayerElementColor is unsupported. see `MeshLoader::on_node_child()`.
}

#[derive(Debug)]
pub struct LayerLoader {
    channel: i32,
    material: Vec<i32>,
    normal: Vec<i32>,
    uv: Vec<i32>,
}

impl LayerLoader {
    pub fn new(channel: i32) -> Self {
        LayerLoader {
            channel: channel,
            material: Default::default(),
            normal: Default::default(),
            uv: Default::default(),
        }
    }

    pub fn from_node_properties(properties: &DelayedProperties) -> Option<Self> {
        if let Some(channel) = properties.iter().next().and_then(|v| v.get_i32()) {
            Some(Self::new(channel))
        } else {
            error!("Invalid property at `/Objects/Geometry(Mesh)/Layer`: type error");
            None
        }
    }
}

impl NodeLoaderCommon for LayerLoader {
    type Target = Option<Layer>;

    fn on_finish(self) -> Result<Self::Target> {
        Ok(Some(Layer {
            channel: self.channel,
            normal: self.normal,
            uv: self.uv,
            material: self.material,
        }))
    }
}

impl<R: Read> NodeLoader<R> for LayerLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(100) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Geometry(Mesh)/Layer` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Geometry(Mesh)/Layer/Version`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "LayerElement" => if let Some((type_name, typed_index)) = try!(LayerElementLoader::new().load(reader)) {
                match type_name.as_ref() {
                    "LayerElementMaterial" => self.material.push(typed_index),
                    "LayerElementNormal" => self.normal.push(typed_index),
                    "LayerElementUV" => self.uv.push(typed_index),
                    val => {
                        error!("Unsupported layer element type: `{}`", val);
                    },
                }
            },
            _ => {
                warn!("Unknown node: `/Objects/Geometry(Mesh)/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}

#[derive(Default)]
struct LayerElementLoader {
    layer_element_type: Option<String>,
    typed_index: Option<i32>,
}

impl LayerElementLoader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl NodeLoaderCommon for LayerElementLoader {
    type Target = Option<(String, i32)>;

    fn on_finish(self) -> Result<Self::Target> {
        if_all_some!{(
            layer_element_type=self.layer_element_type,
            typed_index=self.typed_index,
        ) {
            Ok(Some((layer_element_type, typed_index)))
        } else {
            error!("Required property not found for `/Objects/Geometry(Mesh)/Layer/LayerElement`");
            Ok(None)
        }}
    }
}

impl<R: Read> NodeLoader<R> for LayerElementLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Type" => {
                self.layer_element_type = properties.iter().next().and_then(|p| p.get_string()).map(|v| v.to_owned());
            },
            "TypedIndex" => {
                self.typed_index = properties.iter().next().and_then(|p| p.get_i32());
            },
            _ => {
                warn!("Unknown node: `/Objects/Geometry(Mesh)/Layer/LayerElement/{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
