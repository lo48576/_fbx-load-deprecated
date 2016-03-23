use std::io::Read;
use fbx_binary_reader::EventReader;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use objects::properties::ObjectProperties;
use property::{GenericProperties, GenericPropertiesLoader, OptionalProperties};

#[derive(Debug, Clone)]
pub struct DisplayLayer {
    pub id: i64,
    pub color: [f32; 3],
    pub show: bool,
    pub freeze: bool,
    pub lod_box: bool,
}

#[derive(Debug)]
pub struct DisplayLayerLoader<'a> {
    definitions: &'a Definitions,
    obj_props: &'a ObjectProperties<'a>,
    properties: Option<GenericProperties>,
}

impl<'a> DisplayLayerLoader<'a> {
    pub fn new(definitions: &'a Definitions, obj_props: &'a ObjectProperties<'a>) -> Self {
        DisplayLayerLoader {
            definitions: definitions,
            obj_props: obj_props,
            properties: None,
        }
    }
}

impl<'a> NodeLoaderCommon for DisplayLayerLoader<'a> {
    type Target = Option<DisplayLayer>;

    fn on_finish(mut self) -> Result<Self::Target> {
        let defaults = self.definitions.templates.templates.get(&("CollectionExclusive".to_owned(), "FbxDisplayLayer".to_owned())).map(|t| &t.properties);
        let color = self.properties.get_or_default(defaults, "Color").and_then(|p| p.value.get_vec_f32().into_iter().find(|v| v.len() >= 3).map(|v| [v[0], v[1], v[2]]));
        let show = self.properties.get_or_default(defaults, "Show").and_then(|p| p.value.get_i64().map(|v| v != 0));
        let freeze = self.properties.get_or_default(defaults, "Freeze").and_then(|p| p.value.get_i64().map(|v| v != 0));
        let lod_box = self.properties.get_or_default(defaults, "LODBox").and_then(|p| p.value.get_i64().map(|v| v != 0));
        // FIXME: Use `if_all_some!` macro.
        if_all_some!{(
            color=color,
            show=show,
            freeze=freeze,
            lod_box=lod_box,
        ) {
            Ok(Some(DisplayLayer {
                id: self.obj_props.id,
                color: color,
                show: show,
                freeze: freeze,
                lod_box: lod_box,
            }))
        } else {
            error!("Required property not found for `/Objects/CollectionExclusive(DisplayLayer)`");
            Ok(None)
        }}
    }
}

impl<'a, R: Read> NodeLoader<R> for DisplayLayerLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, .. } = node_info;
        match name.as_ref() {
            "Properties70" => {
                self.properties = Some(try!(GenericPropertiesLoader::new(70).load(reader)));
            },
            _ => {
                warn!("Unknown node: `/Objects/CollectionExclusive(DisplayLayer)/{}`", name);
                try!(ignore_current_node(reader));
            },
        }
        Ok(())
    }
}
