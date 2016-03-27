//! Contains `/Objects/Texture` node-related stuff.

use std::io::Read;
use std::path::PathBuf;
use fbx_binary_reader::EventReader;
use ::separate_name_class;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader, OptionalProperties};


#[derive(Debug, Clone)]
pub struct Texture {
    pub id: i64,
    pub name: String,
    pub media: Option<String>,
    pub filename: PathBuf,
    pub relative_filename: PathBuf,
    pub current_texture_blend_mode: BlendMode,
    pub premultiply_alpha: bool,
    pub uv_set: String,
    pub wrap_mode_u: WrapMode,
    pub wrap_mode_v: WrapMode,
}

#[derive(Debug)]
pub struct TextureLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    properties: Option<GenericProperties>,
    media: Option<String>,
    filename: Option<PathBuf>,
    relative_filename: Option<PathBuf>,
}

impl<'a> TextureLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        TextureLoader {
            definitions: definitions,
            obj_props: obj_props,
            properties: None,
            media: None,
            filename: None,
            relative_filename: None,
        }
    }
}

impl<'a> NodeLoaderCommon for TextureLoader<'a> {
    type Target = Option<Texture>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let defaults = self.definitions.templates.templates.get(&("Texture".to_owned(), "FbxFileTexture".to_owned())).map(|t| &t.properties);
        let current_texture_blend_mode = self.properties.get_or_default(defaults, "CurrentTextureBlendMode").and_then(|p| p.value.get_i64()).and_then(BlendMode::from_i64);
        let premultiply_alpha = self.properties.get_or_default(defaults, "PremultiplyAlpha").and_then(|p| p.value.get_i64().map(|v| v != 0));
        let uv_set = self.properties.get_or_default(defaults, "UVSet").and_then(|p| p.value.get_string().cloned());
        let wrap_mode_u = self.properties.get_or_default(defaults, "WrapModeU").and_then(|p| p.value.get_i64()).and_then(WrapMode::from_i64);
        let wrap_mode_v = self.properties.get_or_default(defaults, "WrapModeV").and_then(|p| p.value.get_i64()).and_then(WrapMode::from_i64);
        if_all_some!{(
            current_texture_blend_mode=current_texture_blend_mode,
            premultiply_alpha=premultiply_alpha,
            uv_set=uv_set,
            wrap_mode_u=wrap_mode_u,
            wrap_mode_v=wrap_mode_v,
            filename=self.filename,
            relative_filename=self.relative_filename,
        ) {
            Ok(Some(Texture {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                media: self.media,
                filename: filename,
                relative_filename: relative_filename,
                current_texture_blend_mode: current_texture_blend_mode,
                premultiply_alpha: premultiply_alpha,
                uv_set: uv_set,
                wrap_mode_u: wrap_mode_u,
                wrap_mode_v: wrap_mode_v,
            }))
        } else {
            error!("Required property not found for `/Objects/Texture`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for TextureLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Type" => {
                match properties.iter().next().and_then(|p| p.get_string()) {
                    Some("TextureVideoClip") => {},
                    Some(t) => {
                        warn!("Maybe unsupported type of `/Objects/Texture` node: type={}", t);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Texture/Type`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "Version" => {
                match properties.iter().next().and_then(|p| p.get_i32()) {
                    Some(202) => {},
                    Some(v) => {
                        warn!("Maybe unsupported version of `/Objects/Texture` node: ver={}", v);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Texture/Version`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "TextureName" => {
                if let Some((name, class)) = properties.iter().next().and_then(|p| p.get_string()).and_then(separate_name_class) {
                    if name != self.obj_props.name || class != self.obj_props.class {
                        warn!("`/Objects/Texture/TextureName` value is different from the name and class at object properties");
                    }
                } else {
                    error!("Invalid property at `/Objects/Texture/TextureName`: type error or invalid format");
                }
                try!(ignore_current_node(reader));
            },
            "Media" => {
                self.media = properties.iter().next().and_then(|p| p.get_string()).and_then(separate_name_class).map(|(name, _class)| name.to_owned());
                try!(ignore_current_node(reader));
            },
            // `FileName`, not `Filename`.
            "FileName" => {
                self.filename = properties.iter().next().and_then(|p| p.get_string().map(|v| v.to_owned().into()));
                try!(ignore_current_node(reader));
            },
            "RelativeFilename" => {
                self.relative_filename = properties.iter().next().and_then(|p| p.get_string().map(|v| v.to_owned().into()));
                try!(ignore_current_node(reader));
            },
            "ModelUVTranslation" | "ModelUVScaling" | "Texture_Alpha_Source" | "Cropping" => {
                // TODO: Unimplemented.
                try!(ignore_current_node(reader));
            },
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            _ => {
                warn!("Unknown node: `/Objects/Texture/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}

/// Texture blend mode.
///
/// See [Help: FbxTexture Class
/// Reference](http://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/class_fbx_texture.html#ae712bb955e55f00dc24eb98c3686dd5a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    Translucent,
    Additive,
    Modulate,
    Modulate2,
    Over,
}

impl BlendMode {
    pub fn from_i64(val: i64) -> Option<Self> {
        match val {
            0 => Some(BlendMode::Translucent),
            1 => Some(BlendMode::Additive),
            2 => Some(BlendMode::Modulate),
            3 => Some(BlendMode::Modulate2),
            4 => Some(BlendMode::Over),
            _ => None,
        }
    }
}

impl Default for BlendMode {
    fn default() -> Self {
        BlendMode::Translucent
    }
}

/// Texture wrap mode.
///
/// See [Help: FbxTexture Class
/// Reference](http://help.autodesk.com/cloudhelp/2016/ENU/FBX-Developer-Help/cpp_ref/class_fbx_texture.html#a889640e63e2e681259ea81061b85143a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WrapMode {
    Repeat,
    Clamp,
}

impl WrapMode {
    pub fn from_i64(val: i64) -> Option<Self> {
        match val {
            0 => Some(WrapMode::Repeat),
            1 => Some(WrapMode::Clamp),
            _ => None,
        }
    }
}

impl Default for WrapMode {
    fn default() -> Self {
        WrapMode::Repeat
    }
}
