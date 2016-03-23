//! Contains `/Objects` node-related stuff.

use std::collections::HashMap;
use std::hash::BuildHasherDefault;
use std::io::Read;
use fbx_binary_reader::EventReader;
use fnv::FnvHasher;
use definitions::Definitions;
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};
use self::properties::ObjectProperties;

pub mod properties;


pub type ObjectsMap<V> = HashMap<i64, V, BuildHasherDefault<FnvHasher>>;

#[derive(Debug, Default, Clone)]
pub struct Objects {
    pub unknown: ObjectsMap<UnknownObject>,
}

macro_rules! implement_method_for_object {
    ($plural:ident, $t:ty, $add_method:ident) => (
        impl Objects {
            pub fn $add_method(&mut self, obj: $t) {
                self.$plural.insert(obj.id, obj);
            }
        }
    )
}
implement_method_for_object!(unknown, UnknownObject, add_unknown);

#[derive(Debug)]
pub struct ObjectsLoader<'a> {
    objects: &'a mut Objects,
    definitions: &'a Definitions,
}

impl<'a> ObjectsLoader<'a> {
    pub fn new(objects: &'a mut Objects, definitions: &'a Definitions) -> Self {
        ObjectsLoader {
            objects: objects,
            definitions: definitions,
        }
    }
}

impl<'a> NodeLoaderCommon for ObjectsLoader<'a> {
    type Target = ();

    fn on_finish(self) -> Result<Self::Target> {
        Ok(())
    }
}

impl<'a, R: Read> NodeLoader<R> for ObjectsLoader<'a> {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        let obj_props = if let Some(val) = ObjectProperties::from_node_properties(properties.iter()) {
            val
        } else {
            try!(ignore_current_node(reader));
            return Ok(());
        };
        warn!("Unknown object node: `/Objects/{}`", name);
        self.objects.unknown.insert(obj_props.id, UnknownObject {
            id: obj_props.id,
            name: obj_props.name.to_owned(),
            class: obj_props.class.to_owned(),
            subclass: obj_props.subclass.to_owned(),
        });
        try!(ignore_current_node(reader));
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
