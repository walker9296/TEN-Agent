//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
use std::collections::HashMap;

use anyhow::Result;

use super::connection::GraphLoc;
use super::node::GraphNode;
use super::{
    Graph, GraphConnection, GraphExposedMessageType, GraphMessageFlow,
    GraphNodeType,
};

impl Graph {
    /// Helper function to flatten a GraphLoc by adding subgraph name prefix
    /// to extension names and validating that no nested subgraphs exist.
    fn flatten_graph_loc_in_subgraph(loc: &mut GraphLoc, subgraph_name: &str) {
        if let Some(ref extension) = loc.extension {
            loc.extension = Some(format!("{}_{}", subgraph_name, extension));
        }

        if loc.subgraph.is_some() {
            panic!(
                "Should not happen, when flattening a subgraph, the subgraphs \
                 inside it should have already been flattened, so we should \
                 not see any subgraph inside."
            );
        }
    }

    /// Updates connection source to use flattened names for subgraph elements.
    fn flatten_connection_source_in_subgraph(
        connection: &mut GraphConnection,
        subgraph_name: &str,
    ) {
        Self::flatten_graph_loc_in_subgraph(&mut connection.loc, subgraph_name);
    }

    /// Updates connection destinations to use flattened names for subgraph
    /// elements.
    fn flatten_connection_destinations_in_subgraph(
        connection: &mut GraphConnection,
        subgraph_name: &str,
    ) {
        let update_destinations = |flows: &mut Vec<GraphMessageFlow>| {
            for flow in flows {
                for dest in &mut flow.dest {
                    Self::flatten_graph_loc_in_subgraph(
                        &mut dest.loc,
                        subgraph_name,
                    );
                }
            }
        };

        if let Some(ref mut cmd) = connection.cmd {
            update_destinations(cmd);
        }
        if let Some(ref mut data) = connection.data {
            update_destinations(data);
        }
        if let Some(ref mut audio_frame) = connection.audio_frame {
            update_destinations(audio_frame);
        }
        if let Some(ref mut video_frame) = connection.video_frame {
            update_destinations(video_frame);
        }
    }

    /// Helper function to resolve subgraph reference to actual extension name.
    /// This function looks up the exposed_messages in the subgraph to find
    /// the corresponding extension for a given message flow.
    fn resolve_subgraph_to_extension(
        subgraph_name: &str,
        subgraph: &Graph,
        flow_name: &str,
        msg_type: GraphExposedMessageType,
    ) -> Result<String> {
        if let Some(exposed_messages) = &subgraph.exposed_messages {
            let matching_exposed = exposed_messages.iter().find(|exposed| {
                exposed.msg_type == msg_type && exposed.name == flow_name
            });

            if let Some(exposed) = matching_exposed {
                if let Some(ref extension_name) = exposed.extension {
                    Ok(format!("{}_{}", subgraph_name, extension_name))
                } else {
                    Err(anyhow::anyhow!(
                        "Exposed message '{}' in subgraph '{}' does not \
                         specify an extension",
                        flow_name,
                        subgraph_name
                    ))
                }
            } else {
                Err(anyhow::anyhow!(
                    "Message '{}' of type '{:?}' is not exposed by subgraph \
                     '{}'",
                    flow_name,
                    msg_type,
                    subgraph_name
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "Subgraph '{}' does not have exposed_messages defined",
                subgraph_name
            ))
        }
    }

    /// Helper function to process message flows and add them to extension_flows
    /// HashMap.
    ///
    /// Since connections originating from a subgraph as source may come from
    /// different extensions within the subgraph, the logic of this function
    /// is not to modify the existing connection, but to generate multiple new
    /// connections originating from extensions within the subgraph.
    fn flatten_connection_source_of_one_type_for_subgraph(
        flows: &[GraphMessageFlow],
        msg_type: GraphExposedMessageType,
        subgraph_name: &str,
        subgraph: &Graph,
        base_connection: &GraphConnection,
        extension_flows: &mut HashMap<String, GraphConnection>,
    ) -> Result<()> {
        for flow in flows {
            let extension_name = Self::resolve_subgraph_to_extension(
                subgraph_name,
                subgraph,
                &flow.name,
                msg_type.clone(),
            )?;

            let entry = extension_flows
                .entry(extension_name.clone())
                .or_insert_with(|| GraphConnection {
                    loc: GraphLoc {
                        app: base_connection.loc.app.clone(),
                        extension: Some(extension_name),
                        subgraph: None,
                    },
                    cmd: None,
                    data: None,
                    audio_frame: None,
                    video_frame: None,
                });

            // Add flow to the appropriate field based on message type
            match msg_type {
                GraphExposedMessageType::CmdOut => {
                    if entry.cmd.is_none() {
                        entry.cmd = Some(Vec::new());
                    }
                    entry.cmd.as_mut().unwrap().push(flow.clone());
                }
                GraphExposedMessageType::DataOut => {
                    if entry.data.is_none() {
                        entry.data = Some(Vec::new());
                    }
                    entry.data.as_mut().unwrap().push(flow.clone());
                }
                GraphExposedMessageType::AudioFrameOut => {
                    if entry.audio_frame.is_none() {
                        entry.audio_frame = Some(Vec::new());
                    }
                    entry.audio_frame.as_mut().unwrap().push(flow.clone());
                }
                GraphExposedMessageType::VideoFrameOut => {
                    if entry.video_frame.is_none() {
                        entry.video_frame = Some(Vec::new());
                    }
                    entry.video_frame.as_mut().unwrap().push(flow.clone());
                }
                _ => {
                    return Err(anyhow::anyhow!(
                        "Unsupported message type for source: {:?}",
                        msg_type
                    ));
                }
            }
        }
        Ok(())
    }

    /// Helper function to handle colon notation in extension or subgraph
    /// fields. Converts "prefix:suffix" to "prefix_suffix".
    fn handle_colon_notation(field: &mut Option<String>) {
        if let Some(ref value) = field {
            if value.contains(':') {
                let parts: Vec<&str> = value.split(':').collect();
                if parts.len() == 2 {
                    *field = Some(format!("{}_{}", parts[0], parts[1]));
                }
            }
        }
    }

    /// Helper function to process a destination location for subgraph
    /// resolution. This function handles both colon notation and subgraph
    /// field resolution.
    fn process_dest_location_for_subgraph_resolution(
        loc: &mut GraphLoc,
        subgraph_mappings: &HashMap<String, Graph>,
        flow_name: &str,
        msg_type: GraphExposedMessageType,
    ) -> Result<()> {
        // Since extension and subgraph fields are mutually exclusive, we can
        // simplify the logic
        if let Some(ref subgraph_name) = loc.subgraph.take() {
            // Handle subgraph field - resolve to actual extension based on
            // exposed_messages
            let subgraph =
                subgraph_mappings.get(subgraph_name).ok_or_else(|| {
                    anyhow::anyhow!("Subgraph '{subgraph_name}' not found")
                })?;

            let extension_name = Self::resolve_subgraph_to_extension(
                subgraph_name,
                subgraph,
                flow_name,
                msg_type,
            )?;

            loc.extension = Some(extension_name);
            // subgraph field is already cleared by take()
        } else {
            // Handle colon notation in extension field
            Self::handle_colon_notation(&mut loc.extension);
        }

        Ok(())
    }

    /// Expands a connection source if it references a subgraph element using
    /// colon notation (e.g., "subgraph_1:ext_c" -> "subgraph_1_ext_c") or
    /// subgraph field. Groups message flows by their resolved extension names
    /// so that flows with the same source extension are combined into a single
    /// connection based on exposed_messages.
    fn flatten_connection_source_for_subgraph(
        connection: &GraphConnection,
        subgraph_mappings: &HashMap<String, Graph>,
    ) -> Result<Vec<GraphConnection>> {
        let mut expanded_connections = Vec::new();
        let mut base_connection = connection.clone();

        // Check if we have a subgraph field that needs to be resolved
        if base_connection.loc.subgraph.is_some() {
            let subgraph_name = base_connection.loc.subgraph.as_ref().unwrap();
            let subgraph =
                subgraph_mappings.get(subgraph_name).ok_or_else(|| {
                    anyhow::anyhow!("Subgraph '{}' not found", subgraph_name)
                })?;

            // Use a HashMap to group flows by their resolved extension names
            let mut extension_flows: HashMap<String, GraphConnection> =
                HashMap::new();

            // Process cmd flows
            if let Some(ref cmd_flows) = base_connection.cmd {
                Self::flatten_connection_source_of_one_type_for_subgraph(
                    cmd_flows,
                    GraphExposedMessageType::CmdOut,
                    subgraph_name,
                    subgraph,
                    &base_connection,
                    &mut extension_flows,
                )?;
            }

            // Process data flows
            if let Some(ref data_flows) = base_connection.data {
                Self::flatten_connection_source_of_one_type_for_subgraph(
                    data_flows,
                    GraphExposedMessageType::DataOut,
                    subgraph_name,
                    subgraph,
                    &base_connection,
                    &mut extension_flows,
                )?;
            }

            // Process audio_frame flows
            if let Some(ref audio_frame_flows) = base_connection.audio_frame {
                Self::flatten_connection_source_of_one_type_for_subgraph(
                    audio_frame_flows,
                    GraphExposedMessageType::AudioFrameOut,
                    subgraph_name,
                    subgraph,
                    &base_connection,
                    &mut extension_flows,
                )?;
            }

            // Process video_frame flows
            if let Some(ref video_frame_flows) = base_connection.video_frame {
                Self::flatten_connection_source_of_one_type_for_subgraph(
                    video_frame_flows,
                    GraphExposedMessageType::VideoFrameOut,
                    subgraph_name,
                    subgraph,
                    &base_connection,
                    &mut extension_flows,
                )?;
            }

            // Convert the HashMap values to a Vec
            expanded_connections.extend(extension_flows.into_values());

            // If no message flows were found, it should not happen
            if expanded_connections.is_empty() {
                panic!(
                    "No message flows found for subgraph: {}",
                    subgraph_name
                );
            }
        } else {
            // Handle colon notation in extension field
            Self::handle_colon_notation(&mut base_connection.loc.extension);
            expanded_connections.push(base_connection);
        }

        Ok(expanded_connections)
    }

    /// Helper function to determine the appropriate GraphExposedMessageType
    /// based on message type string.
    fn get_exposed_message_type_for_dest_subgraph(
        msg_type: &str,
    ) -> Result<GraphExposedMessageType> {
        match msg_type {
            "cmd" => Ok(GraphExposedMessageType::CmdIn),
            "data" => Ok(GraphExposedMessageType::DataIn),
            "audio_frame" => Ok(GraphExposedMessageType::AudioFrameIn),
            "video_frame" => Ok(GraphExposedMessageType::VideoFrameIn),
            _ => Err(anyhow::anyhow!("Unknown message type: {}", msg_type)),
        }
    }

    /// Updates all message flow destinations to convert subgraph references
    /// from colon notation to underscore notation and resolve subgraph field
    /// references using exposed_messages.
    fn flatten_connection_destinations_for_subgraph(
        connection: &mut GraphConnection,
        subgraph_mappings: &HashMap<String, Graph>,
    ) -> Result<()> {
        let update_destinations =
            |flows: &mut Vec<GraphMessageFlow>, msg_type: &str| -> Result<()> {
                for flow in flows {
                    for dest in &mut flow.dest {
                        let exposed_msg_type =
                            Self::get_exposed_message_type_for_dest_subgraph(
                                msg_type,
                            )?;

                        Self::process_dest_location_for_subgraph_resolution(
                            &mut dest.loc,
                            subgraph_mappings,
                            &flow.name,
                            exposed_msg_type,
                        )?;
                    }
                }
                Ok(())
            };

        if let Some(ref mut cmd) = connection.cmd {
            update_destinations(cmd, "cmd")?;
        }
        if let Some(ref mut data) = connection.data {
            update_destinations(data, "data")?;
        }
        if let Some(ref mut audio_frame) = connection.audio_frame {
            update_destinations(audio_frame, "audio_frame")?;
        }
        if let Some(ref mut video_frame) = connection.video_frame {
            update_destinations(video_frame, "video_frame")?;
        }

        Ok(())
    }

    /// Applies properties from subgraph node reference to a flattened extension
    /// node based on exposed_properties mapping.
    fn apply_subgraph_properties_to_extension(
        flattened_node: &mut GraphNode,
        sub_node: &GraphNode,
        subgraph_node: &GraphNode,
        subgraph: &Graph,
    ) -> Result<()> {
        // Apply properties from subgraph node reference based on
        // exposed_properties mapping
        if let Some(serde_json::Value::Object(ref_obj)) =
            &subgraph_node.property
        {
            // Process each property specified in the subgraph node
            for (property_alias, property_value) in ref_obj {
                // Find the corresponding exposed property by alias
                if let Some(exposed_properties) = &subgraph.exposed_properties {
                    if let Some(exposed_prop) = exposed_properties
                        .iter()
                        .find(|ep| &ep.alias == property_alias)
                    {
                        // Check if this exposed property applies to the current
                        // extension
                        if let Some(ref target_extension) =
                            exposed_prop.extension
                        {
                            if target_extension == &sub_node.name {
                                // Initialize property object if it doesn't
                                // exist
                                if flattened_node.property.is_none() {
                                    flattened_node.property =
                                        Some(serde_json::Value::Object(
                                            serde_json::Map::new(),
                                        ));
                                }

                                // Apply the property value to the target
                                // property name
                                if let Some(serde_json::Value::Object(
                                    node_obj,
                                )) = &mut flattened_node.property
                                {
                                    node_obj.insert(
                                        exposed_prop.name.clone(),
                                        property_value.clone(),
                                    );
                                }
                            }
                        } else {
                            panic!(
                                "Property '{property_alias}' specified in \
                                 subgraph node '{}' is not exposed by the \
                                 subgraph",
                                subgraph_node.name
                            );
                        }
                    } else {
                        return Err(anyhow::anyhow!(
                            "Property '{}' specified in subgraph node '{}' is \
                             not exposed by the subgraph",
                            property_alias,
                            subgraph_node.name
                        ));
                    }
                } else {
                    return Err(anyhow::anyhow!(
                        "Subgraph '{}' does not have exposed_properties \
                         defined, but properties are specified in the \
                         subgraph node",
                        subgraph_node.name
                    ));
                }
            }
        }

        Ok(())
    }

    /// Helper function to add internal connections from a subgraph with proper
    /// name prefixing.
    fn add_internal_connections_from_subgraph(
        subgraph: &Graph,
        subgraph_name: &str,
        flattened_connections: &mut Vec<GraphConnection>,
    ) {
        if let Some(sub_connections) = &subgraph.connections {
            for connection in sub_connections {
                let mut flattened_connection = connection.clone();

                // Update extension names in the connection source
                Self::flatten_connection_source_in_subgraph(
                    &mut flattened_connection,
                    subgraph_name,
                );

                // Update extension names in all message flows
                Self::flatten_connection_destinations_in_subgraph(
                    &mut flattened_connection,
                    subgraph_name,
                );

                flattened_connections.push(flattened_connection);
            }
        }
    }

    /// Helper function to process extension nodes from a subgraph and add them
    /// to the flattened nodes list with proper name prefixing and property
    /// application.
    fn process_extension_nodes_from_subgraph(
        subgraph_nodes: &[GraphNode],
        subgraph_name: &str,
        subgraph_node: &GraphNode,
        subgraph: &Graph,
        flattened_nodes: &mut Vec<GraphNode>,
    ) -> Result<()> {
        for sub_node in subgraph_nodes {
            let mut flattened_node = sub_node.clone();
            // Add subgraph name as prefix
            flattened_node.name =
                format!("{}_{}", subgraph_name, sub_node.name);

            // Apply properties from subgraph node reference based on
            // exposed_properties mapping
            Self::apply_subgraph_properties_to_extension(
                &mut flattened_node,
                sub_node,
                subgraph_node,
                subgraph,
            )?;

            flattened_nodes.push(flattened_node);
        }
        Ok(())
    }

    /// Helper function to process a loaded subgraph and integrate it into the
    /// flattened structure.
    fn process_loaded_subgraph<F>(
        subgraph_node: &GraphNode,
        loaded_subgraph: Graph,
        subgraph_loader: &F,
        flattened_nodes: &mut Vec<GraphNode>,
        flattened_connections: &mut Vec<GraphConnection>,
        subgraph_mappings: &mut HashMap<String, Graph>,
    ) -> Result<()>
    where
        F: Fn(&str) -> Result<Graph>,
    {
        // First, recursively flatten any nested subgraphs within this subgraph.
        // This ensures depth-first processing.
        let flattened_subgraph = Self::flatten_subgraph_recursively(
            loaded_subgraph,
            subgraph_loader,
        )?;

        subgraph_mappings
            .insert(subgraph_node.name.clone(), flattened_subgraph.clone());

        // Flatten subgraph nodes (now all should be extensions after recursive
        // flattening)
        for sub_node in &flattened_subgraph.nodes {
            if sub_node.type_ != GraphNodeType::Extension {
                return Err(anyhow::anyhow!(
                    "Unexpected non-extension node '{}' in flattened subgraph \
                     '{}'",
                    sub_node.name,
                    subgraph_node.name
                ));
            }
        }

        // Process all extension nodes from the subgraph
        Self::process_extension_nodes_from_subgraph(
            &flattened_subgraph.nodes,
            &subgraph_node.name,
            subgraph_node,
            &flattened_subgraph,
            flattened_nodes,
        )?;

        // Add internal connections from subgraph
        Self::add_internal_connections_from_subgraph(
            &flattened_subgraph,
            &subgraph_node.name,
            flattened_connections,
        );

        Ok(())
    }

    /// Helper function to process a single subgraph node and add its flattened
    /// content to the output collections.
    fn process_subgraph_node<F>(
        subgraph_node: &GraphNode,
        subgraph_loader: &F,
        flattened_nodes: &mut Vec<GraphNode>,
        flattened_connections: &mut Vec<GraphConnection>,
        subgraph_mappings: &mut HashMap<String, Graph>,
    ) -> Result<()>
    where
        F: Fn(&str) -> Result<Graph>,
    {
        let source_uri =
            subgraph_node.source_uri.as_ref().ok_or_else(|| {
                anyhow::anyhow!(
                    "Subgraph node '{}' must have source_uri",
                    subgraph_node.name
                )
            })?;

        let subgraph = subgraph_loader(source_uri)?;

        Self::process_loaded_subgraph(
            subgraph_node,
            subgraph,
            subgraph_loader,
            flattened_nodes,
            flattened_connections,
            subgraph_mappings,
        )
    }

    /// Helper function to process connections from a graph using subgraph
    /// mappings.
    fn process_graph_connections(
        connections: &[GraphConnection],
        subgraph_mappings: &HashMap<String, Graph>,
        flattened_connections: &mut Vec<GraphConnection>,
    ) -> Result<()> {
        for connection in connections {
            // Expand connection source if it references a subgraph element
            let expanded_connections =
                Self::flatten_connection_source_for_subgraph(
                    connection,
                    subgraph_mappings,
                )?;

            for mut flattened_connection in expanded_connections {
                // Update all message flow destinations
                Self::flatten_connection_destinations_for_subgraph(
                    &mut flattened_connection,
                    subgraph_mappings,
                )?;

                flattened_connections.push(flattened_connection);
            }
        }
        Ok(())
    }

    /// Helper function that contains the common logic for flattening a graph's
    /// nodes and connections.
    fn flatten_graph_internal<F>(
        graph: &Graph,
        subgraph_loader: &F,
        flattened_nodes: &mut Vec<GraphNode>,
        flattened_connections: &mut Vec<GraphConnection>,
        subgraph_mappings: &mut HashMap<String, Graph>,
    ) -> Result<()>
    where
        F: Fn(&str) -> Result<Graph>,
    {
        // Process all nodes in the graph
        for node in &graph.nodes {
            match node.type_ {
                GraphNodeType::Extension => {
                    flattened_nodes.push(node.clone());
                }
                GraphNodeType::Subgraph => {
                    Self::process_subgraph_node(
                        node,
                        subgraph_loader,
                        flattened_nodes,
                        flattened_connections,
                        subgraph_mappings,
                    )?;
                }
            }
        }

        // Process connections from the graph
        if let Some(connections) = &graph.connections {
            Self::process_graph_connections(
                connections,
                subgraph_mappings,
                flattened_connections,
            )?;
        }

        Ok(())
    }

    /// Flattens a graph containing subgraph nodes into a regular graph
    /// structure with only extension nodes. This process converts subgraph
    /// references into their constituent extensions with prefixed names and
    /// merges all connections.
    pub fn flatten<F>(&self, subgraph_loader: F) -> Result<Graph>
    where
        F: Fn(&str) -> Result<Graph>,
    {
        let mut flattened_nodes = Vec::new();
        let mut flattened_connections = Vec::new();
        let mut subgraph_mappings: HashMap<String, Graph> = HashMap::new();

        Self::flatten_graph_internal(
            self,
            &subgraph_loader,
            &mut flattened_nodes,
            &mut flattened_connections,
            &mut subgraph_mappings,
        )?;

        Ok(Graph {
            nodes: flattened_nodes,
            connections: if flattened_connections.is_empty() {
                None
            } else {
                Some(flattened_connections)
            },
            // exposed_messages and exposed_properties are discarded during
            // flattening
            exposed_messages: None,
            exposed_properties: None,
        })
    }

    /// Recursively flattens a subgraph to handle nested subgraphs.
    /// This is a separate function to avoid infinite recursion in type
    /// inference.
    fn flatten_subgraph_recursively<F>(
        subgraph: Graph,
        subgraph_loader: &F,
    ) -> Result<Graph>
    where
        F: Fn(&str) -> Result<Graph>,
    {
        // Check if this subgraph contains any nested subgraphs
        let has_nested_subgraphs = subgraph
            .nodes
            .iter()
            .any(|node| node.type_ == GraphNodeType::Subgraph);

        if !has_nested_subgraphs {
            // No nested subgraphs, return as-is
            return Ok(subgraph);
        }

        // This subgraph has nested subgraphs, so we need to flatten them
        let mut flattened_nodes = Vec::new();
        let mut flattened_connections = Vec::new();
        let mut subgraph_mappings = HashMap::new();

        Self::flatten_graph_internal(
            &subgraph,
            subgraph_loader,
            &mut flattened_nodes,
            &mut flattened_connections,
            &mut subgraph_mappings,
        )?;

        // Update exposed_messages to reflect the flattened structure
        let updated_exposed_messages = if let Some(exposed_messages) =
            &subgraph.exposed_messages
        {
            let mut updated = Vec::new();
            for exposed in exposed_messages {
                if let Some(ref extension_name) = exposed.extension {
                    // Check if this extension is actually a subgraph that was
                    // flattened
                    if let Some(flattened_subgraph) =
                        subgraph_mappings.get(extension_name)
                    {
                        // This exposed message points to a subgraph, we need to
                        // resolve it further
                        if let Some(nested_exposed_messages) =
                            &flattened_subgraph.exposed_messages
                        {
                            for nested_exposed in nested_exposed_messages {
                                if nested_exposed.msg_type == exposed.msg_type
                                    && nested_exposed.name == exposed.name
                                {
                                    if let Some(ref nested_extension) =
                                        nested_exposed.extension
                                    {
                                        // Create a new exposed message with the
                                        // flattened extension name
                                        let mut new_exposed = exposed.clone();
                                        new_exposed.extension = Some(format!(
                                            "{}_{}",
                                            extension_name, nested_extension
                                        ));
                                        updated.push(new_exposed);
                                    }
                                }
                            }
                        }
                    } else {
                        // This is a regular extension, keep as-is
                        updated.push(exposed.clone());
                    }
                } else {
                    // No extension specified, keep as-is
                    updated.push(exposed.clone());
                }
            }
            if updated.is_empty() {
                None
            } else {
                Some(updated)
            }
        } else {
            None
        };

        Ok(Graph {
            nodes: flattened_nodes,
            connections: if flattened_connections.is_empty() {
                None
            } else {
                Some(flattened_connections)
            },
            exposed_messages: updated_exposed_messages,
            exposed_properties: None,
        })
    }
}
