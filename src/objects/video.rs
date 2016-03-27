//! Contains `/Objects/Texture` node-related stuff.

use std::io::Read;
use std::path::PathBuf;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{FormatConvert, NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader, OptionalProperties};


#[derive(Debug, Clone)]
pub struct Video<I: Clone> {
    pub id: i64,
    pub name: String,
    pub path: PathBuf,
    pub use_mip_map: bool,
    pub filename: PathBuf,
    pub relative_filename: PathBuf,
    pub content: Option<I>,
}

#[derive(Debug)]
pub struct VideoLoader<'a, C: 'a + FormatConvert> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    converter: &'a mut C,
    properties: Option<GenericProperties>,
    use_mip_map: Option<bool>,
    filename: Option<PathBuf>,
    relative_filename: Option<PathBuf>,
    content: Option<C::ImageResult>,
}

impl<'a, C: 'a + FormatConvert> VideoLoader<'a, C> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>, converter: &'a mut C) -> Self {
        VideoLoader {
            definitions: definitions,
            obj_props: obj_props,
            converter: converter,
            properties: None,
            use_mip_map: None,
            filename: None,
            relative_filename: None,
            content: None,
        }
    }
}

impl<'a, C: 'a + FormatConvert> NodeLoaderCommon for VideoLoader<'a, C> {
    type Target = Option<Video<C::ImageResult>>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let defaults = self.definitions.templates.templates.get(&("Video".to_owned(), "FbxVideo".to_owned())).map(|t| &t.properties);
        let path = self.properties.get_or_default(defaults, "Path").and_then(|p| p.value.get_string().cloned()).map(Into::into);
        if_all_some!{(
            path=path,
            use_mip_map=self.use_mip_map,
            filename=self.filename,
            relative_filename=self.relative_filename,
        ) {
            Ok(Some(Video {
                id: self.obj_props.id,
                name: self.obj_props.name.to_owned(),
                path: path,
                use_mip_map: use_mip_map,
                filename: filename,
                relative_filename: relative_filename,
                content: self.content,
            }))
        } else {
            error!("Required property not found for `/Objects/Video`");
            Ok(None)
        }}
    }
}

impl<'a, C: FormatConvert, R: Read> NodeLoader<R> for VideoLoader<'a, C> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "Type" => {
                match properties.iter().next().and_then(|p| p.get_string()) {
                    Some("Clip") => {},
                    Some(t) => {
                        warn!("Maybe unsupported type of `/Objects/Video` node: type={}", t);
                    },
                    None => {
                        error!("Invalid proprety at `/Objects/Video/Type`: type error");
                    },
                }
                try!(ignore_current_node(reader));
            },
            "UseMipMap" => {
                self.use_mip_map = properties.iter().next().and_then(|p| p.get_i32()).map(|v| v != 0);
                try!(ignore_current_node(reader));
            },
            // `Filename`, not `FileName`.
            "Filename" => {
                self.filename = properties.iter().next().and_then(|p| p.get_string().map(|v| v.to_owned().into()));
                try!(ignore_current_node(reader));
            },
            "RelativeFilename" => {
                self.relative_filename = properties.iter().next().and_then(|p| p.get_string().map(|v| v.to_owned().into()));
                try!(ignore_current_node(reader));
            },
            "Content" => {
                let &mut VideoLoader { ref filename, ref mut converter, ref mut content, .. } = self;
                if let Some(ref filename) = *filename {
                    *content = properties.iter().next().and_then(|p| p.get_binary()).map(|v| converter.binary_to_image(v, filename));
                } else {
                    error!("`/Objects/Video(Clip)/Filename` should be read before `/Objects/Video(Clip)/Content`");
                }
                try!(ignore_current_node(reader));
            },
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            _ => {
                warn!("Unknown node: `/Objects/Video/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
