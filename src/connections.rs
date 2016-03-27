//! Contains a type related to connections between objects.

use std::io::Read;
use fbx_binary_reader::{EventReader, DelayedProperties};
use error::Result;
use node_loader::{NodeLoader, NodeLoaderCommon, RawNodeInfo, ignore_current_node};

/// A connection between two objects.
#[derive(Debug, Clone)]
pub struct Connection {
    pub parent: i64,
    pub child: i64,
    pub attribute: Option<String>,
    pub parent_is_property: bool,
    pub child_is_property: bool,
}

impl Connection {
    /// Load `Connectio` node data from node properties.
    pub fn from_node_properties(properties: &DelayedProperties) -> Option<Self> {
        let mut iter = properties.iter();
        let connection_type = iter.next().and_then(|p| p.get_string())
            .and_then(|type_name| match type_name {
                "OO" => Some((false, false)),
                "OP" => Some((false, true)),
                "PO" => Some((true, false)),
                "PP" => Some((true, true)),
                val => {
                    warn!("Invalid connection type: `{}`", val);
                    None
                },
            });
        let child = iter.next().and_then(|p| p.get_i64());
        let parent = iter.next().and_then(|p| p.get_i64());
        let attr_name = iter.next().and_then(|p| p.get_string());
        if let (Some((child_is_prop, parent_is_prop)), Some(child), Some(parent)) = (connection_type, child, parent) {
            Some(Connection {
                parent: parent,
                child: child,
                attribute: attr_name.map(|v| v.to_owned()),
                parent_is_property: parent_is_prop,
                child_is_property: child_is_prop,
            })
        } else {
            None
        }
    }

    pub fn has_attribute<S: AsRef<str>>(&self, name: S) -> bool {
        self.attribute.as_ref().map_or(false, |n| n == name.as_ref())
    }
}

#[derive(Debug, Default)]
pub struct ConnectionsLoader {
    connections: Vec<Connection>,
}

impl ConnectionsLoader {
    pub fn new() -> Self {
        Default::default()
    }
}

impl NodeLoaderCommon for ConnectionsLoader {
    type Target = Vec<Connection>;

    fn on_finish(mut self) -> Result<Self::Target> {
        self.connections.shrink_to_fit();
        Ok(self.connections)
    }
}

impl<R: Read> NodeLoader<R> for ConnectionsLoader {
    fn on_child_node(&mut self, reader: &mut EventReader<R>, node_info: RawNodeInfo) -> Result<()> {
        let RawNodeInfo { name, properties } = node_info;
        match name.as_ref() {
            "C" => if let Some(c) = Connection::from_node_properties(&properties) {
                self.connections.push(c);
            },
            _ => {
                warn!("Unknown node: `{}`", name);
            },
        }
        try!(ignore_current_node(reader));
        Ok(())
    }
}
