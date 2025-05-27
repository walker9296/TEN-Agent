//
// Copyright © 2025 Agora
// This file is part of TEN Framework, an open source project.
// Licensed under the Apache License, Version 2.0, with certain conditions.
// Refer to the "LICENSE" file in the root directory for more information.
//
#[cfg(test)]
mod tests {
    use std::{collections::HashMap, path::Path, str::FromStr};

    use uuid::Uuid;

    use ten_rust::{
        base_dir_pkg_info::PkgsInfoInApp,
        graph::{graph_info::GraphInfo, Graph},
        pkg_info::get_app_installed_pkgs,
    };

    #[test]
    fn test_graph_check_extension_not_installed_1() {
        let mut graphs_cache: HashMap<Uuid, GraphInfo> = HashMap::new();

        let app_dir = "tests/test_data/graph_check_extension_not_installed_1";
        let pkgs_info_in_app = get_app_installed_pkgs(
            Path::new(app_dir),
            true,
            &mut Some(&mut graphs_cache),
        )
        .unwrap();
        assert!(!pkgs_info_in_app.is_empty());

        let (_, graph_info) = graphs_cache.into_iter().next().unwrap();

        let graph = &graph_info.graph;

        let mut pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();
        pkgs_cache.insert(app_dir.to_string(), pkgs_info_in_app);

        let result = graph.check(&Some(app_dir.to_string()), &pkgs_cache);
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_graph_check_extension_not_installed_2() {
        let mut graphs_cache: HashMap<Uuid, GraphInfo> = HashMap::new();

        let app_dir = "tests/test_data/graph_check_extension_not_installed_2";
        let pkgs_info_in_app = get_app_installed_pkgs(
            Path::new(app_dir),
            true,
            &mut Some(&mut graphs_cache),
        )
        .unwrap();
        assert!(!pkgs_info_in_app.is_empty());

        let (_, graph_info) = graphs_cache.into_iter().next().unwrap();
        let graph = &graph_info.graph;

        let mut pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();
        pkgs_cache.insert(app_dir.to_string(), pkgs_info_in_app);

        let result = graph.check(&Some(app_dir.to_string()), &pkgs_cache);
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_graph_check_predefined_graph_success() {
        let mut graphs_cache: HashMap<Uuid, GraphInfo> = HashMap::new();

        let app_dir = "tests/test_data/graph_check_predefined_graph_success";
        let pkgs_info_in_app = get_app_installed_pkgs(
            Path::new(app_dir),
            true,
            &mut Some(&mut graphs_cache),
        )
        .unwrap();
        assert!(!pkgs_info_in_app.is_empty());

        let (_, graph_info) = graphs_cache.into_iter().next().unwrap();
        let graph = &graph_info.graph;

        let mut pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();
        pkgs_cache.insert(app_dir.to_string(), pkgs_info_in_app);

        let result = graph.check(&Some(app_dir.to_string()), &pkgs_cache);
        eprintln!("result: {result:?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_check_all_msgs_schema_incompatible() {
        let mut graphs_cache: HashMap<Uuid, GraphInfo> = HashMap::new();

        let app_dir =
            "tests/test_data/graph_check_all_msgs_schema_incompatible";
        let pkgs_info_in_app = get_app_installed_pkgs(
            Path::new(app_dir),
            true,
            &mut Some(&mut graphs_cache),
        )
        .unwrap();
        assert!(!pkgs_info_in_app.is_empty());

        let (_, graph_info) = graphs_cache.into_iter().next().unwrap();
        let graph = &graph_info.graph;

        let mut pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();
        pkgs_cache.insert(app_dir.to_string(), pkgs_info_in_app);

        let result = graph.check(&Some(app_dir.to_string()), &pkgs_cache);
        assert!(result.is_err());
        println!("Error: {:?}", result.err().unwrap());
    }

    #[test]
    fn test_graph_check_single_app() {
        let app_dir = "tests/test_data/graph_check_single_app";
        let pkgs_info_in_app = get_app_installed_pkgs(
            Path::new(app_dir),
            true,
            &mut Some(&mut HashMap::new()),
        )
        .unwrap();
        assert!(!pkgs_info_in_app.is_empty());

        let graph_json_str =
            include_str!("../test_data/graph_check_single_app/graph.json");
        let graph = Graph::from_str(graph_json_str).unwrap();

        let mut pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();
        pkgs_cache.insert(app_dir.to_string(), pkgs_info_in_app);

        // The schema of 'ext_c' is not found, but it's OK because we only check
        // for the app 'http://localhost:8001'.
        let result =
            graph.check_for_single_app(&Some(app_dir.to_string()), &pkgs_cache);
        eprintln!("result: {result:?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_check_builtin_extension() {
        let app_dir = "tests/test_data/graph_check_builtin_extension";
        let pkgs_info_in_app = get_app_installed_pkgs(
            Path::new(app_dir),
            true,
            &mut Some(&mut HashMap::new()),
        )
        .unwrap();
        assert!(!pkgs_info_in_app.is_empty());

        let graph_json_str = include_str!(
            "../test_data/graph_check_builtin_extension/graph.json"
        );
        let graph = Graph::from_str(graph_json_str).unwrap();

        let mut pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();
        pkgs_cache.insert(app_dir.to_string(), pkgs_info_in_app);

        let result =
            graph.check_for_single_app(&Some(app_dir.to_string()), &pkgs_cache);
        eprintln!("result: {result:?}");
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_check_subgraph_reference_missing() {
        // Test that subgraph references in connections are validated
        let graph_json = r#"
        {
            "nodes": [
                {
                    "type": "extension",
                    "name": "ext_a",
                    "addon": "addon_a",
                    "extension_group": "some_group"
                }
            ],
            "connections": [
                {
                    "extension": "ext_a",
                    "cmd": [
                        {
                            "name": "test_cmd",
                            "dest": [
                                {
                                    "extension": "subgraph_1:ext_b"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

        let graph = Graph::from_str(graph_json).unwrap();
        let pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();

        let result = graph.check(&None, &pkgs_cache);
        assert!(result.is_err());

        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("subgraph 'subgraph_1'"));
        assert!(error_msg.contains("is not defined in nodes"));
    }

    #[test]
    fn test_graph_check_subgraph_reference_valid() {
        // Test that valid subgraph references pass validation
        let graph_json = r#"
        {
            "nodes": [
                {
                    "type": "extension",
                    "name": "ext_a",
                    "addon": "addon_a",
                    "extension_group": "some_group"
                },
                {
                    "type": "subgraph",
                    "name": "subgraph_1",
                    "source_uri": "./subgraph.json"
                }
            ],
            "connections": [
                {
                    "extension": "ext_a",
                    "cmd": [
                        {
                            "name": "test_cmd",
                            "dest": [
                                {
                                    "extension": "subgraph_1:ext_b"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

        let graph = Graph::from_str(graph_json).unwrap();
        let pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();

        let result = graph.check(&None, &pkgs_cache);
        // This should fail due to missing extension installation, but not due
        // to subgraph reference
        assert!(result.is_err());

        let error_msg = result.err().unwrap().to_string();
        eprintln!("error_msg: {error_msg}");
        // Should not contain subgraph error
        assert!(!error_msg.contains("subgraph 'subgraph_1'"));
    }

    #[test]
    fn test_graph_check_direct_subgraph_reference_missing() {
        // Test that direct subgraph references in connections are validated
        let graph_json = r#"
        {
            "nodes": [
                {
                    "type": "extension",
                    "name": "ext_a",
                    "addon": "addon_a",
                    "extension_group": "some_group"
                }
            ],
            "connections": [
                {
                    "subgraph": "missing_subgraph",
                    "cmd": [
                        {
                            "name": "test_cmd",
                            "dest": [
                                {
                                    "extension": "ext_a"
                                }
                            ]
                        }
                    ]
                }
            ]
        }
        "#;

        let graph = Graph::from_str(graph_json).unwrap();
        let pkgs_cache: HashMap<String, PkgsInfoInApp> = HashMap::new();

        let result = graph.check(&None, &pkgs_cache);
        assert!(result.is_err());
        let error_msg = result.err().unwrap().to_string();
        assert!(error_msg.contains("subgraph 'missing_subgraph'"));
        assert!(error_msg.contains("is not defined in nodes"));
    }
}
